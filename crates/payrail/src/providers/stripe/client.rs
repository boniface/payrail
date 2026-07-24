use std::borrow::Cow;

#[cfg(feature = "telemetry")]
use crate::emit_provider_request_result;
use crate::{
    CheckoutUiMode, CreatePaymentRequest, MerchantReference, Metadata, NextAction, PaymentError,
    PaymentEvent, PaymentMethod, PaymentProvider, PaymentSession, PaymentStatusResponse,
    ProviderErrorDetails, ProviderReference, RefundRequest, RefundResponse, StablecoinAsset,
    StablecoinPaymentMethod, WebhookEventId, WebhookRequest,
};
#[cfg(feature = "fraud")]
use crate::{
    FraudEvent, FraudEventType, FraudProvider, FraudProviderReference, RiskAssessment,
    RiskDecision, RiskLevel, RiskReason, RiskReasonCode, RiskScore,
};
use secrecy::ExposeSecret;
use url::Url;

use super::{
    config::StripeConfig,
    mapper::{map_event_type, map_payment_status, map_refund_status},
    models::{StripeCheckoutSession, StripeEvent, StripeRefund},
    webhook::verify_signature,
};

type StripeFormPair<'a> = (Cow<'a, str>, Cow<'a, str>);
type StripeForm<'a> = Vec<StripeFormPair<'a>>;

/// Stripe `PayRail` connector.
#[derive(Debug, Clone)]
pub struct StripeConnector {
    config: StripeConfig,
    client: reqwest::Client,
}

impl StripeConnector {
    /// Creates a Stripe connector.
    ///
    /// # Errors
    ///
    /// Returns an error when the HTTP client cannot be built.
    pub fn new(config: StripeConfig) -> Result<Self, PaymentError> {
        let client = reqwest::Client::builder()
            .user_agent("payrail-rs/0.2 (+https://github.com/boniface/payrail)")
            .timeout(config.request_timeout_value())
            .build()?;
        Ok(Self { config, client })
    }

    fn endpoint(&self, path: &str) -> Result<Url, PaymentError> {
        self.config
            .api_base_url()
            .join(path)
            .map_err(|error| PaymentError::InvalidConfiguration(error.to_string()))
    }

    async fn parse_response<T: serde::de::DeserializeOwned>(
        &self,
        operation: &'static str,
        response: reqwest::Response,
    ) -> Result<T, PaymentError> {
        let status = response.status();
        #[cfg(not(feature = "telemetry"))]
        let _ = operation;
        #[cfg(feature = "telemetry")]
        emit_provider_request_result(
            &PaymentProvider::Stripe,
            operation,
            status.as_u16(),
            status.is_success(),
        );
        if status.is_success() {
            return Ok(response.json::<T>().await?);
        }

        let request_id = response
            .headers()
            .get("request-id")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);

        Err(PaymentError::ProviderDetails {
            details: ProviderErrorDetails {
                provider: PaymentProvider::Stripe,
                status: status.as_u16(),
                code: None,
                request_id,
                message: "stripe request failed".to_owned(),
            },
        })
    }

    async fn payment_intent_for_refund(
        &self,
        provider_reference: &ProviderReference,
    ) -> Result<String, PaymentError> {
        if !provider_reference.as_str().starts_with("cs_") {
            return Ok(provider_reference.as_str().to_owned());
        }

        let session = self.checkout_session(provider_reference).await?;
        session.payment_intent.ok_or_else(|| {
            PaymentError::InvalidConfiguration(
                "stripe checkout session has no payment_intent to refund".to_owned(),
            )
        })
    }

    async fn checkout_session(
        &self,
        provider_reference: &ProviderReference,
    ) -> Result<StripeCheckoutSession, PaymentError> {
        let path = format!("/v1/checkout/sessions/{}", provider_reference.as_str());
        self.parse_response(
            "checkout_session_retrieve",
            self.client
                .get(self.endpoint(&path)?)
                .bearer_auth(self.config.secret_key().expose_secret())
                .send()
                .await?,
        )
        .await
    }

    const fn supports_stablecoin(method: &StablecoinPaymentMethod) -> bool {
        matches!(
            method.preferred_asset.as_ref(),
            None | Some(StablecoinAsset::Usdc)
        )
    }

    fn verified_stripe_event(
        &self,
        request: WebhookRequest<'_>,
    ) -> Result<StripeEvent, PaymentError> {
        let secret = self.config.webhook_secret_value().ok_or_else(|| {
            PaymentError::InvalidConfiguration("missing stripe webhook secret".to_owned())
        })?;
        let signature = request
            .headers
            .get("stripe-signature")
            .and_then(|value| value.to_str().ok())
            .ok_or(PaymentError::WebhookVerificationFailed)?;
        tracing::debug!(
            "payrail.provider" = "stripe",
            "payrail.operation" = "parse_webhook",
            "payrail.payload_len" = request.payload.len(),
            "verifying webhook signature"
        );
        verify_signature(request.payload, signature, secret)?;
        Ok(serde_json::from_slice(request.payload)?)
    }
}

impl StripeConnector {
    #[must_use]
    pub const fn provider_id(&self) -> PaymentProvider {
        PaymentProvider::Stripe
    }

    /// Creates a Stripe checkout payment.
    ///
    /// # Errors
    ///
    /// Returns an error when the request cannot be mapped or Stripe rejects the request.
    pub async fn create_payment(
        &self,
        request: CreatePaymentRequest,
    ) -> Result<PaymentSession, PaymentError> {
        let payment_method_type = match request.payment_method() {
            PaymentMethod::Card(_) => "card",
            PaymentMethod::Stablecoin(method) => {
                if !Self::supports_stablecoin(method) {
                    return Err(PaymentError::UnsupportedPaymentMethod(
                        "stripe stablecoin checkout currently supports USDC only".to_owned(),
                    ));
                }

                if request.amount().currency().as_str() != "USD" {
                    return Err(PaymentError::UnsupportedCurrency(
                        request.amount().currency().clone(),
                    ));
                }
                "crypto"
            }
            PaymentMethod::Crypto(_) | PaymentMethod::PayPal(_) | PaymentMethod::MobileMoney(_) => {
                return Err(PaymentError::UnsupportedPaymentMethod(
                    "stripe only supports card and stablecoin routes".to_owned(),
                ));
            }
        };
        let checkout_ui_mode = request.checkout_ui_mode();
        let form = checkout_session_form(&request, payment_method_type)?;
        let mut builder = self
            .client
            .post(self.endpoint("/v1/checkout/sessions")?)
            .bearer_auth(self.config.secret_key().expose_secret())
            .form(&form);
        if let Some(key) = request.idempotency_key() {
            builder = builder.header("Idempotency-Key", key.as_str());
        }

        tracing::debug!(
            "payrail.provider" = "stripe",
            "payrail.operation" = "create_payment",
            "payrail.payment_method" = payment_method_type,
            "payrail.has_idempotency_key" = request.idempotency_key().is_some(),
            "sending provider request"
        );
        let session: StripeCheckoutSession = self
            .parse_response("create_payment", builder.send().await?)
            .await?;
        let StripeCheckoutSession {
            id,
            client_secret,
            payment_intent: _,
            url,
            payment_status,
            status,
        } = session;
        let reference = request.into_reference();
        let next_action = checkout_next_action(checkout_ui_mode, url.as_deref(), client_secret)?;

        PaymentSession::new(
            PaymentProvider::Stripe,
            ProviderReference::new(&id)?,
            reference,
            map_payment_status(status.as_deref(), payment_status.as_deref()),
            next_action,
        )
    }

    /// Gets the Stripe payment status.
    ///
    /// # Errors
    ///
    /// Returns an error when Stripe rejects the request or the response cannot be parsed.
    pub async fn get_payment_status(
        &self,
        provider_reference: &ProviderReference,
    ) -> Result<PaymentStatusResponse, PaymentError> {
        tracing::debug!(
            "payrail.provider" = "stripe",
            "payrail.operation" = "get_payment_status",
            "sending provider request"
        );
        let session = self.checkout_session(provider_reference).await?;

        Ok(PaymentStatusResponse {
            provider: PaymentProvider::Stripe,
            provider_reference: ProviderReference::new(session.id)?,
            status: map_payment_status(
                session.status.as_deref(),
                session.payment_status.as_deref(),
            ),
        })
    }

    /// Refunds a Stripe payment.
    ///
    /// # Errors
    ///
    /// Returns an error when the request cannot be mapped or Stripe rejects the request.
    pub async fn refund_payment(
        &self,
        request: RefundRequest,
    ) -> Result<RefundResponse, PaymentError> {
        let payment_intent = self
            .payment_intent_for_refund(&request.provider_reference)
            .await?;
        let mut form: Vec<(&'static str, Cow<'_, str>)> =
            Vec::with_capacity(1 + usize::from(request.amount.is_some()));
        form.push(("payment_intent", Cow::Owned(payment_intent)));
        if let Some(amount) = request.amount.as_ref() {
            form.push(("amount", Cow::Owned(amount.amount().value().to_string())));
        }
        tracing::debug!(
            "payrail.provider" = "stripe",
            "payrail.operation" = "refund_payment",
            "sending provider request"
        );
        let refund: StripeRefund = self
            .parse_response(
                "refund_payment",
                self.client
                    .post(self.endpoint("/v1/refunds")?)
                    .bearer_auth(self.config.secret_key().expose_secret())
                    .header("Idempotency-Key", request.idempotency_key.as_str())
                    .form(&form)
                    .send()
                    .await?,
            )
            .await?;

        Ok(RefundResponse {
            provider: PaymentProvider::Stripe,
            provider_reference: ProviderReference::new(refund.id)?,
            status: map_refund_status(refund.status.as_deref()),
        })
    }

    /// Parses and verifies a Stripe webhook.
    ///
    /// # Errors
    ///
    /// Returns an error when verification fails or the payload is invalid.
    pub async fn parse_webhook(
        &self,
        request: WebhookRequest<'_>,
    ) -> Result<PaymentEvent, PaymentError> {
        let event = self.verified_stripe_event(request)?;
        let (event_type, status) = map_event_type(&event.event_type);
        let (event_type, status) = dispute_payment_event_type(&event, event_type, status);
        let provider_reference = event
            .data
            .object
            .get("payment_intent")
            .or_else(|| event.data.object.get("id"))
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                PaymentError::InvalidWebhookPayload("missing provider reference".to_owned())
            })?;
        let merchant_reference = event
            .data
            .object
            .get("client_reference_id")
            .and_then(serde_json::Value::as_str)
            .map(MerchantReference::new)
            .transpose()?;

        Ok(PaymentEvent {
            id: Some(WebhookEventId::new(event.id)?),
            provider: PaymentProvider::Stripe,
            provider_reference: ProviderReference::new(provider_reference)?,
            merchant_reference,
            status,
            amount: None,
            event_type,
            message: None,
        })
    }

    /// Parses and verifies a Stripe fraud or dispute webhook.
    ///
    /// # Errors
    ///
    /// Returns an error when verification fails, the payload is invalid, or the event type is not
    /// fraud/dispute related.
    #[cfg(feature = "fraud")]
    pub async fn parse_fraud_webhook(
        &self,
        request: WebhookRequest<'_>,
    ) -> Result<FraudEvent, PaymentError> {
        let event = self.verified_stripe_event(request)?;
        stripe_fraud_event(event)
    }
}

fn dispute_payment_event_type(
    event: &StripeEvent,
    event_type: crate::PaymentEventType,
    status: crate::PaymentStatus,
) -> (crate::PaymentEventType, crate::PaymentStatus) {
    if event.event_type != "charge.dispute.closed" {
        return (event_type, status);
    }

    match string_field(&event.data.object, "status") {
        Some("won") => (
            crate::PaymentEventType::DisputeWon,
            crate::PaymentStatus::Succeeded,
        ),
        Some("lost") => (
            crate::PaymentEventType::DisputeLost,
            crate::PaymentStatus::Failed,
        ),
        Some(_) | None => (
            crate::PaymentEventType::DisputeUpdated,
            crate::PaymentStatus::Processing,
        ),
    }
}

#[cfg(feature = "fraud")]
fn stripe_fraud_event(event: StripeEvent) -> Result<FraudEvent, PaymentError> {
    let (event_type, decision, score, level) =
        stripe_fraud_event_type(&event.event_type, &event.data.object)?;
    let provider_reference = string_field(&event.data.object, "id")
        .map(FraudProviderReference::new)
        .transpose()?;
    let payment_reference = string_field(&event.data.object, "payment_intent")
        .or_else(|| string_field(&event.data.object, "charge"))
        .map(ProviderReference::new)
        .transpose()?;
    let merchant_reference = string_field(&event.data.object, "client_reference_id")
        .or_else(|| metadata_string_field(&event.data.object, "client_reference_id"))
        .or_else(|| metadata_string_field(&event.data.object, "merchant_reference"))
        .map(MerchantReference::new)
        .transpose()?;
    let assessment = RiskAssessment::new(decision)
        .with_provider(FraudProvider::StripeRadar)
        .with_score(RiskScore::new(score).expect("stripe fraud score should be valid"))
        .with_level(level)
        .with_reason(RiskReason::new(RiskReasonCode::ProviderRule));
    let mut fraud_event = FraudEvent::new(FraudProvider::StripeRadar, event_type)
        .with_id(WebhookEventId::new(event.id)?)
        .with_assessment(assessment);

    if let Some(reference) = provider_reference {
        fraud_event = fraud_event.with_provider_reference(reference);
    }
    if let Some(reference) = payment_reference {
        fraud_event = fraud_event.with_payment_reference(PaymentProvider::Stripe, reference);
    }
    if let Some(reference) = merchant_reference {
        fraud_event = fraud_event.with_merchant_reference(reference);
    }

    Ok(fraud_event)
}

#[cfg(feature = "fraud")]
fn stripe_fraud_event_type(
    event_type: &str,
    object: &serde_json::Value,
) -> Result<(FraudEventType, RiskDecision, u16, RiskLevel), PaymentError> {
    match event_type {
        "radar.early_fraud_warning.created" => Ok((
            FraudEventType::EarlyFraudWarningCreated,
            RiskDecision::Review,
            700,
            RiskLevel::High,
        )),
        "review.opened" => Ok((
            FraudEventType::ReviewOpened,
            RiskDecision::Review,
            650,
            RiskLevel::High,
        )),
        "review.closed" => Ok(match string_field(object, "reason") {
            Some("approved") => (
                FraudEventType::ReviewApproved,
                RiskDecision::Allow,
                250,
                RiskLevel::Medium,
            ),
            Some("refunded_as_fraud") | Some("refunded") | Some("disputed") => (
                FraudEventType::ReviewRejected,
                RiskDecision::Reject,
                850,
                RiskLevel::Critical,
            ),
            Some(_) | None => (
                FraudEventType::ReviewRejected,
                RiskDecision::Review,
                650,
                RiskLevel::High,
            ),
        }),
        "charge.dispute.created" => Ok((
            FraudEventType::DisputeOpened,
            RiskDecision::Review,
            850,
            RiskLevel::Critical,
        )),
        "charge.dispute.updated" => Ok((
            FraudEventType::DisputeUpdated,
            RiskDecision::Review,
            750,
            RiskLevel::Critical,
        )),
        "charge.dispute.closed" => Ok(match string_field(object, "status") {
            Some("won") => (
                FraudEventType::DisputeWon,
                RiskDecision::Allow,
                250,
                RiskLevel::Medium,
            ),
            Some("lost") => (
                FraudEventType::DisputeLost,
                RiskDecision::Reject,
                900,
                RiskLevel::Critical,
            ),
            Some(_) | None => (
                FraudEventType::DisputeUpdated,
                RiskDecision::Review,
                750,
                RiskLevel::Critical,
            ),
        }),
        _ => Err(PaymentError::InvalidWebhookPayload(format!(
            "unsupported stripe fraud event type: {event_type}"
        ))),
    }
}

fn string_field<'a>(object: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    object.get(key).and_then(serde_json::Value::as_str)
}

#[cfg(feature = "fraud")]
fn metadata_string_field<'a>(object: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    object
        .get("metadata")
        .and_then(|metadata| metadata.get(key))
        .and_then(serde_json::Value::as_str)
}

fn checkout_session_form<'a>(
    request: &'a CreatePaymentRequest,
    payment_method_type: &'static str,
) -> Result<StripeForm<'a>, PaymentError> {
    let return_url = request
        .return_url()
        .ok_or(PaymentError::MissingRequiredField("return_url"))?;
    let description = request.description().unwrap_or("PayRail payment");
    let payment_metadata = payment_metadata_for_request(request);
    let customer_email = request.customer().and_then(|customer| customer.email());
    let mut form = Vec::with_capacity(
        9 + request.metadata().len()
            + payment_metadata.len()
            + usize::from(customer_email.is_some()),
    );
    form.push((Cow::Borrowed("mode"), Cow::Borrowed("payment")));
    match request.checkout_ui_mode() {
        CheckoutUiMode::Hosted => {
            let cancel_url = request
                .cancel_url()
                .ok_or(PaymentError::MissingRequiredField("cancel_url"))?;
            form.push((
                Cow::Borrowed("success_url"),
                Cow::Borrowed(return_url.as_str()),
            ));
            form.push((
                Cow::Borrowed("cancel_url"),
                Cow::Borrowed(cancel_url.as_str()),
            ));
        }
        CheckoutUiMode::Custom | CheckoutUiMode::Elements => {
            form.push((Cow::Borrowed("ui_mode"), Cow::Borrowed("custom")));
            form.push((
                Cow::Borrowed("return_url"),
                Cow::Borrowed(return_url.as_str()),
            ));
        }
    }
    form.extend([
        (
            Cow::Borrowed("client_reference_id"),
            Cow::Borrowed(request.reference().as_str()),
        ),
        (
            Cow::Borrowed("payment_method_types[0]"),
            Cow::Borrowed(payment_method_type),
        ),
        (Cow::Borrowed("line_items[0][quantity]"), Cow::Borrowed("1")),
        (
            Cow::Borrowed("line_items[0][price_data][currency]"),
            Cow::Owned(request.amount().currency().as_str().to_ascii_lowercase()),
        ),
        (
            Cow::Borrowed("line_items[0][price_data][unit_amount]"),
            Cow::Owned(request.amount().amount().value().to_string()),
        ),
        (
            Cow::Borrowed("line_items[0][price_data][product_data][name]"),
            Cow::Borrowed(description),
        ),
    ]);
    push_metadata(&mut form, "metadata", request.metadata());
    push_metadata(&mut form, "payment_intent_data[metadata]", payment_metadata);
    if let Some(email) = customer_email {
        form.push((Cow::Borrowed("customer_email"), Cow::Borrowed(email)));
    }
    Ok(form)
}

fn payment_metadata_for_request(request: &CreatePaymentRequest) -> &Metadata {
    if request.payment_metadata().is_empty() {
        request.metadata()
    } else {
        request.payment_metadata()
    }
}

fn push_metadata<'a>(form: &mut StripeForm<'a>, prefix: &str, metadata: &'a Metadata) {
    form.extend(metadata.iter().map(|(key, value)| {
        (
            Cow::Owned(format!("{prefix}[{key}]")),
            Cow::Borrowed(value.as_str()),
        )
    }));
}

fn checkout_next_action(
    checkout_ui_mode: CheckoutUiMode,
    url: Option<&str>,
    client_secret: Option<String>,
) -> Result<Option<NextAction>, PaymentError> {
    match checkout_ui_mode {
        CheckoutUiMode::Hosted => url
            .map(Url::parse)
            .transpose()
            .map(|url| url.map(|url| NextAction::RedirectToUrl { url }))
            .map_err(|error| PaymentError::InvalidUrl(error.to_string())),
        CheckoutUiMode::Custom | CheckoutUiMode::Elements => {
            let client_secret = client_secret.ok_or_else(|| {
                PaymentError::InvalidConfiguration(
                    "stripe custom checkout session has no client_secret".to_owned(),
                )
            })?;
            Ok(Some(NextAction::EmbeddedCheckout { client_secret }))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use crate::{CheckoutUiMode, Customer, Money, PaymentMethod};
    use hmac::{Hmac, KeyInit, Mac};
    use secrecy::SecretString;
    use serde_json::json;
    use sha2::Sha256;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_string_contains, header, method, path},
    };

    use super::*;

    #[tokio::test]
    async fn create_checkout_session_sends_required_request() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/checkout/sessions"))
            .and(header("authorization", "Bearer sk_test_payrail"))
            .and(header("idempotency-key", "ORDER-1:create"))
            .and(body_string_contains("payment_method_types%5B0%5D=card"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "cs_test_123",
                "url": "https://checkout.stripe.com/c/payrail",
                "payment_status": "unpaid",
                "status": "open"
            })))
            .mount(&server)
            .await;
        let config = StripeConfig::new(SecretString::from("sk_test_payrail".to_owned()))
            .expect("config should be valid")
            .api_base(Url::parse(&server.uri()).expect("mock url should parse"));
        let connector = StripeConnector::new(config).expect("connector should build");
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(1_000, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::card())
            .return_url("https://example.com/success")
            .expect("return url should be valid")
            .cancel_url("https://example.com/cancel")
            .expect("cancel url should be valid")
            .idempotency_key("ORDER-1:create")
            .expect("key should be valid")
            .build()
            .expect("request should be valid");

        let session = connector
            .create_payment(request)
            .await
            .expect("session should be created");

        assert_eq!(session.provider_reference.as_str(), "cs_test_123");
        assert!(matches!(
            session.next_action,
            Some(NextAction::RedirectToUrl { .. })
        ));
    }

    #[tokio::test]
    async fn create_custom_checkout_session_sends_custom_request() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/checkout/sessions"))
            .and(header("authorization", "Bearer sk_test_payrail"))
            .and(body_string_contains("ui_mode=custom"))
            .and(body_string_contains("return_url="))
            .and(body_string_contains("payment_method_types%5B0%5D=card"))
            .and(body_string_contains("metadata%5Btenant_id%5D=tenant_123"))
            .and(body_string_contains(
                "payment_intent_data%5Bmetadata%5D%5Btenant_id%5D=tenant_123",
            ))
            .and(body_string_contains("customer_email=buyer%40example.com"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "cs_test_custom_123",
                "client_secret": "cs_test_custom_123_secret_payrail",
                "payment_status": "unpaid",
                "status": "open"
            })))
            .mount(&server)
            .await;
        let config = StripeConfig::new(SecretString::from("sk_test_payrail".to_owned()))
            .expect("config should be valid")
            .api_base(Url::parse(&server.uri()).expect("mock url should parse"));
        let connector = StripeConnector::new(config).expect("connector should build");
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(1_000, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .customer(Customer::new().with_email("buyer@example.com"))
            .payment_method(PaymentMethod::card())
            .checkout_ui_mode(CheckoutUiMode::Custom)
            .return_url("https://example.com/stripe/return?session_id={CHECKOUT_SESSION_ID}")
            .expect("return url should be valid")
            .metadata("tenant_id", "tenant_123")
            .build()
            .expect("request should be valid");

        let session = connector
            .create_payment(request)
            .await
            .expect("session should be created");

        assert_eq!(session.provider_reference.as_str(), "cs_test_custom_123");
        assert_eq!(
            session.next_action,
            Some(NextAction::EmbeddedCheckout {
                client_secret: "cs_test_custom_123_secret_payrail".to_owned()
            })
        );
        let requests = server
            .received_requests()
            .await
            .expect("request recording should be enabled");
        let body = String::from_utf8_lossy(&requests[0].body);
        assert!(!body.contains("cancel_url="));
        assert!(!body.contains("success_url="));
    }

    #[tokio::test]
    async fn legacy_elements_mode_sends_custom_ui_mode() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/checkout/sessions"))
            .and(body_string_contains("ui_mode=custom"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "cs_test_elements_123",
                "client_secret": "cs_test_elements_123_secret_payrail",
                "payment_status": "unpaid",
                "status": "open"
            })))
            .mount(&server)
            .await;
        let config = StripeConfig::new(SecretString::from("sk_test_payrail".to_owned()))
            .expect("config should be valid")
            .api_base(Url::parse(&server.uri()).expect("mock url should parse"));
        let connector = StripeConnector::new(config).expect("connector should build");
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(1_000, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::card())
            .checkout_ui_mode(CheckoutUiMode::Elements)
            .return_url("https://example.com/stripe/return?session_id={CHECKOUT_SESSION_ID}")
            .expect("return url should be valid")
            .build()
            .expect("request should be valid");

        let session = connector
            .create_payment(request)
            .await
            .expect("session should be created");

        assert_eq!(session.provider_reference.as_str(), "cs_test_elements_123");
    }

    #[tokio::test]
    async fn custom_checkout_uses_explicit_payment_metadata_when_present() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/checkout/sessions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "cs_test_custom_123",
                "client_secret": "cs_test_custom_123_secret_payrail",
                "payment_status": "unpaid",
                "status": "open"
            })))
            .mount(&server)
            .await;
        let config = StripeConfig::new(SecretString::from("sk_test_payrail".to_owned()))
            .expect("config should be valid")
            .api_base(Url::parse(&server.uri()).expect("mock url should parse"));
        let connector = StripeConnector::new(config).expect("connector should build");
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(1_000, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::card())
            .checkout_ui_mode(CheckoutUiMode::Custom)
            .return_url("https://example.com/stripe/return?session_id={CHECKOUT_SESSION_ID}")
            .expect("return url should be valid")
            .metadata("tenant_id", "tenant_session")
            .payment_metadata("tenant_id", "tenant_payment")
            .build()
            .expect("request should be valid");

        let _session = connector
            .create_payment(request)
            .await
            .expect("session should be created");
        let requests = server
            .received_requests()
            .await
            .expect("request recording should be enabled");
        let body = String::from_utf8_lossy(&requests[0].body);

        assert!(body.contains("metadata%5Btenant_id%5D=tenant_session"));
        assert!(body.contains("payment_intent_data%5Bmetadata%5D%5Btenant_id%5D=tenant_payment"));
        assert!(!body.contains("payment_intent_data%5Bmetadata%5D%5Btenant_id%5D=tenant_session"));
    }

    #[tokio::test]
    async fn embedded_checkout_requires_return_url() {
        let server = MockServer::start().await;
        let config = StripeConfig::new(SecretString::from("sk_test_payrail".to_owned()))
            .expect("config should be valid")
            .api_base(Url::parse(&server.uri()).expect("mock url should parse"));
        let connector = StripeConnector::new(config).expect("connector should build");
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(1_000, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::card())
            .checkout_ui_mode(CheckoutUiMode::Custom)
            .build()
            .expect("request should be valid");

        let error = connector
            .create_payment(request)
            .await
            .expect_err("missing return url should fail");

        assert!(matches!(
            error,
            PaymentError::MissingRequiredField("return_url")
        ));
    }

    #[tokio::test]
    async fn hosted_checkout_still_requires_cancel_url() {
        let server = MockServer::start().await;
        let config = StripeConfig::new(SecretString::from("sk_test_payrail".to_owned()))
            .expect("config should be valid")
            .api_base(Url::parse(&server.uri()).expect("mock url should parse"));
        let connector = StripeConnector::new(config).expect("connector should build");
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(1_000, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::card())
            .return_url("https://example.com/success")
            .expect("return url should be valid")
            .build()
            .expect("request should be valid");

        let error = connector
            .create_payment(request)
            .await
            .expect_err("missing cancel url should fail");

        assert!(matches!(
            error,
            PaymentError::MissingRequiredField("cancel_url")
        ));
    }

    #[tokio::test]
    async fn embedded_checkout_requires_client_secret() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/checkout/sessions"))
            .and(body_string_contains("ui_mode=custom"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "cs_test_custom_123",
                "payment_status": "unpaid",
                "status": "open"
            })))
            .mount(&server)
            .await;
        let config = StripeConfig::new(SecretString::from("sk_test_payrail".to_owned()))
            .expect("config should be valid")
            .api_base(Url::parse(&server.uri()).expect("mock url should parse"));
        let connector = StripeConnector::new(config).expect("connector should build");
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(1_000, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::card())
            .checkout_ui_mode(CheckoutUiMode::Custom)
            .return_url("https://example.com/stripe/return")
            .expect("return url should be valid")
            .build()
            .expect("request should be valid");

        let error = connector
            .create_payment(request)
            .await
            .expect_err("missing client secret should fail");

        assert!(matches!(error, PaymentError::InvalidConfiguration(_)));
    }

    #[tokio::test]
    async fn status_refund_and_webhook_are_normalized() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/checkout/sessions/cs_test_123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "cs_test_123",
                "payment_intent": "pi_123",
                "url": null,
                "payment_status": "paid",
                "status": "complete"
            })))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v1/refunds"))
            .and(header("idempotency-key", "ORDER-1:refund"))
            .and(body_string_contains("payment_intent=pi_123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "re_123",
                "status": "succeeded"
            })))
            .mount(&server)
            .await;
        let config = StripeConfig::new(SecretString::from("sk_test_payrail".to_owned()))
            .expect("config should be valid")
            .webhook_secret(Some(SecretString::from("whsec_test".to_owned())))
            .api_base(Url::parse(&server.uri()).expect("mock url should parse"));
        let connector = StripeConnector::new(config).expect("connector should build");
        let status = connector
            .get_payment_status(
                &ProviderReference::new("cs_test_123").expect("reference should be valid"),
            )
            .await
            .expect("status should parse");
        let refund = connector
            .refund_payment(RefundRequest {
                provider: PaymentProvider::Stripe,
                provider_reference: ProviderReference::new("cs_test_123")
                    .expect("reference should be valid"),
                idempotency_key: crate::IdempotencyKey::new("ORDER-1:refund")
                    .expect("key should be valid"),
                amount: None,
                reason: None,
            })
            .await
            .expect("refund should parse");
        let payload = br#"{
            "id":"evt_123",
            "type":"payment_intent.succeeded",
            "data":{"object":{"id":"pi_123","client_reference_id":"ORDER-1"}}
        }"#;
        let event = connector
            .parse_webhook(WebhookRequest::new(payload, signed_headers(payload)))
            .await
            .expect("webhook should parse");

        assert_eq!(status.status, crate::PaymentStatus::Succeeded);
        assert_eq!(refund.status, crate::PaymentStatus::Refunded);
        assert_eq!(event.provider_reference.as_str(), "pi_123");
    }

    #[tokio::test]
    async fn dispute_webhook_is_visible_as_payment_event() {
        let config = StripeConfig::new(SecretString::from("sk_test_payrail".to_owned()))
            .expect("config should be valid")
            .webhook_secret(Some(SecretString::from("whsec_test".to_owned())));
        let connector = StripeConnector::new(config).expect("connector should build");
        let payload = br#"{
            "id":"evt_dispute_123",
            "type":"charge.dispute.closed",
            "data":{"object":{"id":"du_123","payment_intent":"pi_123","status":"lost","metadata":{"merchant_reference":"ORDER-1"}}}
        }"#;

        let event = connector
            .parse_webhook(WebhookRequest::new(payload, signed_headers(payload)))
            .await
            .expect("dispute webhook should parse");

        assert_eq!(event.provider_reference.as_str(), "pi_123");
        assert_eq!(event.event_type(), crate::PaymentEventType::DisputeLost);
        assert_eq!(event.status(), crate::PaymentStatus::Failed);
    }

    #[cfg(feature = "fraud")]
    #[tokio::test]
    async fn fraud_webhook_normalizes_stripe_dispute() {
        let config = StripeConfig::new(SecretString::from("sk_test_payrail".to_owned()))
            .expect("config should be valid")
            .webhook_secret(Some(SecretString::from("whsec_test".to_owned())));
        let connector = StripeConnector::new(config).expect("connector should build");
        let payload = br#"{
            "id":"evt_fraud_123",
            "type":"charge.dispute.created",
            "data":{"object":{"id":"du_123","payment_intent":"pi_123","metadata":{"merchant_reference":"ORDER-1"}}}
        }"#;

        let event = connector
            .parse_fraud_webhook(WebhookRequest::new(payload, signed_headers(payload)))
            .await
            .expect("fraud webhook should parse");

        assert_eq!(event.event_type(), crate::FraudEventType::DisputeOpened);
        assert_eq!(
            event
                .provider_reference()
                .expect("provider reference should exist")
                .as_str(),
            "du_123"
        );
        assert_eq!(
            event
                .payment_provider_reference()
                .expect("payment reference should exist")
                .as_str(),
            "pi_123"
        );
        assert_eq!(
            event
                .merchant_reference()
                .expect("merchant reference should exist")
                .as_str(),
            "ORDER-1"
        );
        assert_eq!(
            event
                .assessment()
                .expect("assessment should exist")
                .decision(),
            crate::RiskDecision::Review
        );
    }

    #[cfg(feature = "fraud")]
    #[tokio::test]
    async fn fraud_webhook_rejects_non_fraud_event_type() {
        let config = StripeConfig::new(SecretString::from("sk_test_payrail".to_owned()))
            .expect("config should be valid")
            .webhook_secret(Some(SecretString::from("whsec_test".to_owned())));
        let connector = StripeConnector::new(config).expect("connector should build");
        let payload = br#"{
            "id":"evt_payment_123",
            "type":"payment_intent.succeeded",
            "data":{"object":{"id":"pi_123"}}
        }"#;

        let error = connector
            .parse_fraud_webhook(WebhookRequest::new(payload, signed_headers(payload)))
            .await
            .expect_err("payment event should not parse as fraud");

        assert!(matches!(error, PaymentError::InvalidWebhookPayload(_)));
    }

    fn signed_headers(payload: &[u8]) -> http::HeaderMap {
        let timestamp = time::OffsetDateTime::now_utc().unix_timestamp().to_string();
        let mut signed_payload = Vec::with_capacity(timestamp.len() + 1 + payload.len());
        signed_payload.extend_from_slice(timestamp.as_bytes());
        signed_payload.push(b'.');
        signed_payload.extend_from_slice(payload);
        let mut mac =
            Hmac::<Sha256>::new_from_slice(b"whsec_test").expect("hmac should initialize");
        mac.update(&signed_payload);
        let signature = format!(
            "t={timestamp},v1={}",
            hex_for_test(&mac.finalize().into_bytes())
        );
        let mut headers = http::HeaderMap::new();
        headers.insert(
            "stripe-signature",
            signature.parse().expect("signature header should parse"),
        );
        headers
    }

    fn hex_for_test(bytes: &[u8]) -> String {
        let mut output = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            let _ = write!(output, "{byte:02x}");
        }
        output
    }
}
