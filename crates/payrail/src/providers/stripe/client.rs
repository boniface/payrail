use std::borrow::Cow;

use crate::{
    CheckoutUiMode, CreatePaymentRequest, MerchantReference, NextAction, PaymentError,
    PaymentEvent, PaymentMethod, PaymentProvider, PaymentSession, PaymentStatusResponse,
    ProviderErrorDetails, ProviderReference, RefundRequest, RefundResponse, StablecoinAsset,
    StablecoinPaymentMethod, WebhookEventId, WebhookRequest,
};
use secrecy::ExposeSecret;
use url::Url;

use super::{
    config::StripeConfig,
    mapper::{map_event_type, map_payment_status, map_refund_status},
    models::{StripeCheckoutSession, StripeEvent, StripeRefund},
    webhook::verify_signature,
};

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
            .user_agent("payrail-rs/0.1 (+https://github.com/boniface/payrail)")
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
        response: reqwest::Response,
    ) -> Result<T, PaymentError> {
        let status = response.status();
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
            provider = "stripe",
            operation = "create_payment",
            payment_method = payment_method_type,
            has_idempotency_key = request.idempotency_key().is_some(),
            "sending provider request"
        );
        let session: StripeCheckoutSession = self.parse_response(builder.send().await?).await?;
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
            provider = "stripe",
            operation = "get_payment_status",
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
            provider = "stripe",
            operation = "refund_payment",
            has_partial_amount = request.amount.is_some(),
            "sending provider request"
        );
        let refund: StripeRefund = self
            .parse_response(
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
        let secret = self.config.webhook_secret_value().ok_or_else(|| {
            PaymentError::InvalidConfiguration("missing stripe webhook secret".to_owned())
        })?;
        let signature = request
            .headers
            .get("stripe-signature")
            .and_then(|value| value.to_str().ok())
            .ok_or(PaymentError::WebhookVerificationFailed)?;
        tracing::debug!(
            provider = "stripe",
            operation = "parse_webhook",
            payload_len = request.payload.len(),
            "verifying webhook signature"
        );
        verify_signature(request.payload, signature, secret)?;
        let event: StripeEvent = serde_json::from_slice(request.payload)?;
        let (event_type, status) = map_event_type(&event.event_type);
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
}

fn checkout_session_form<'a>(
    request: &'a CreatePaymentRequest,
    payment_method_type: &'static str,
) -> Result<Vec<(&'static str, Cow<'a, str>)>, PaymentError> {
    let return_url = request
        .return_url()
        .ok_or(PaymentError::MissingRequiredField("return_url"))?;
    let description = request.description().unwrap_or("PayRail payment");
    let mut form = Vec::with_capacity(9);
    form.push(("mode", Cow::Borrowed("payment")));
    match request.checkout_ui_mode() {
        CheckoutUiMode::Hosted => {
            let cancel_url = request
                .cancel_url()
                .ok_or(PaymentError::MissingRequiredField("cancel_url"))?;
            form.push(("success_url", Cow::Borrowed(return_url.as_str())));
            form.push(("cancel_url", Cow::Borrowed(cancel_url.as_str())));
        }
        CheckoutUiMode::Elements => {
            form.push(("ui_mode", Cow::Borrowed("elements")));
            form.push(("return_url", Cow::Borrowed(return_url.as_str())));
        }
    }
    form.extend([
        (
            "client_reference_id",
            Cow::Borrowed(request.reference().as_str()),
        ),
        (
            "payment_method_types[0]",
            Cow::Borrowed(payment_method_type),
        ),
        ("line_items[0][quantity]", Cow::Borrowed("1")),
        (
            "line_items[0][price_data][currency]",
            Cow::Owned(request.amount().currency().as_str().to_ascii_lowercase()),
        ),
        (
            "line_items[0][price_data][unit_amount]",
            Cow::Owned(request.amount().amount().value().to_string()),
        ),
        (
            "line_items[0][price_data][product_data][name]",
            Cow::Borrowed(description),
        ),
    ]);
    Ok(form)
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
        CheckoutUiMode::Elements => {
            let client_secret = client_secret.ok_or_else(|| {
                PaymentError::InvalidConfiguration(
                    "stripe embedded checkout session has no client_secret".to_owned(),
                )
            })?;
            Ok(Some(NextAction::EmbeddedCheckout { client_secret }))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use crate::{CheckoutUiMode, Money, PaymentMethod};
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
    async fn create_embedded_checkout_session_sends_elements_request() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/checkout/sessions"))
            .and(header("authorization", "Bearer sk_test_payrail"))
            .and(body_string_contains("ui_mode=elements"))
            .and(body_string_contains("return_url="))
            .and(body_string_contains("payment_method_types%5B0%5D=card"))
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
        assert_eq!(
            session.next_action,
            Some(NextAction::EmbeddedCheckout {
                client_secret: "cs_test_elements_123_secret_payrail".to_owned()
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
            .checkout_ui_mode(CheckoutUiMode::Elements)
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
            .and(body_string_contains("ui_mode=elements"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "cs_test_elements_123",
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
        let timestamp = time::OffsetDateTime::now_utc().unix_timestamp().to_string();
        let mut signed_payload = Vec::new();
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
        let event = connector
            .parse_webhook(WebhookRequest::new(payload, headers))
            .await
            .expect("webhook should parse");

        assert_eq!(status.status, crate::PaymentStatus::Succeeded);
        assert_eq!(refund.status, crate::PaymentStatus::Refunded);
        assert_eq!(event.provider_reference.as_str(), "pi_123");
    }

    fn hex_for_test(bytes: &[u8]) -> String {
        let mut output = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            let _ = write!(output, "{byte:02x}");
        }
        output
    }
}
