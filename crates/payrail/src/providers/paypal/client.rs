use crate::{
    CapturablePaymentConnector, CaptureRequest, CaptureResponse, CreatePaymentRequest, NextAction,
    PaymentConnector, PaymentError, PaymentEvent, PaymentMethod, PaymentProvider, PaymentSession,
    PaymentStatusResponse, ProviderErrorDetails, ProviderReference, RefundRequest, RefundResponse,
    WebhookRequest,
};
use async_trait::async_trait;
use url::Url;

use super::{
    auth::TokenCache,
    config::PayPalConfig,
    mapper::map_order_status,
    models::{PayPalOrder, VerifyWebhookSignatureRequest, VerifyWebhookSignatureResponse},
    orders::{approval_url, create_order_body},
    webhook::parse_event,
};

/// PayPal PayRail connector.
#[derive(Debug)]
pub struct PayPalConnector {
    config: PayPalConfig,
    client: reqwest::Client,
    tokens: TokenCache,
}

impl PayPalConnector {
    /// Creates a PayPal connector.
    ///
    /// # Errors
    ///
    /// Returns an error when the HTTP client cannot be built.
    pub fn new(config: PayPalConfig) -> Result<Self, PaymentError> {
        let client = reqwest::Client::builder()
            .user_agent("payrail-rs/0.1 (+https://github.com/boniface/payrail)")
            .timeout(config.request_timeout_value())
            .build()?;
        Ok(Self {
            config,
            client,
            tokens: TokenCache::default(),
        })
    }

    fn endpoint(&self, path: &str) -> Result<Url, PaymentError> {
        self.config
            .base_url_value()
            .join(path)
            .map_err(|error| PaymentError::InvalidConfiguration(error.to_string()))
    }

    async fn access_token(&self) -> Result<String, PaymentError> {
        self.tokens.access_token(&self.client, &self.config).await
    }

    async fn parse_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, PaymentError> {
        let status = response.status();
        if status.is_success() {
            return Ok(response.json::<T>().await?);
        }

        Err(PaymentError::ProviderDetails {
            details: ProviderErrorDetails {
                provider: PaymentProvider::PayPal,
                status: status.as_u16(),
                code: None,
                request_id: None,
                message: "paypal request failed".to_owned(),
            },
        })
    }

    async fn verify_webhook_signature(
        &self,
        request: &WebhookRequest<'_>,
    ) -> Result<(), PaymentError> {
        let webhook_id = self.config.webhook_id_value().ok_or_else(|| {
            PaymentError::InvalidConfiguration("missing paypal webhook id".to_owned())
        })?;
        let body = VerifyWebhookSignatureRequest {
            auth_algo: header_value(&request.headers, "paypal-auth-algo")?,
            cert_url: header_value(&request.headers, "paypal-cert-url")?,
            transmission_id: header_value(&request.headers, "paypal-transmission-id")?,
            transmission_sig: header_value(&request.headers, "paypal-transmission-sig")?,
            transmission_time: header_value(&request.headers, "paypal-transmission-time")?,
            webhook_id,
            webhook_event: serde_json::from_slice(request.payload)?,
        };
        let token = self.access_token().await?;
        let response: VerifyWebhookSignatureResponse = self
            .parse_response(
                self.client
                    .post(self.endpoint("/v1/notifications/verify-webhook-signature")?)
                    .bearer_auth(token)
                    .json(&body)
                    .send()
                    .await?,
            )
            .await?;

        if response.verification_status == "SUCCESS" {
            return Ok(());
        }

        Err(PaymentError::WebhookVerificationFailed)
    }
}

fn header_value<'a>(headers: &'a http::HeaderMap, name: &str) -> Result<&'a str, PaymentError> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .ok_or(PaymentError::WebhookVerificationFailed)
}

#[async_trait]
impl PaymentConnector for PayPalConnector {
    fn provider(&self) -> PaymentProvider {
        PaymentProvider::PayPal
    }

    async fn create_payment(
        &self,
        request: CreatePaymentRequest,
    ) -> Result<PaymentSession, PaymentError> {
        if !matches!(request.payment_method(), PaymentMethod::PayPal(_)) {
            return Err(PaymentError::UnsupportedPaymentMethod(
                "paypal connector only supports paypal routes".to_owned(),
            ));
        }

        let token = self.access_token().await?;
        let mut builder = self
            .client
            .post(self.endpoint("/v2/checkout/orders")?)
            .bearer_auth(token)
            .json(&create_order_body(&request));
        if let Some(key) = request.idempotency_key() {
            builder = builder.header("PayPal-Request-Id", key.as_str());
        }

        tracing::debug!(
            provider = "paypal",
            operation = "create_payment",
            has_idempotency_key = request.idempotency_key().is_some(),
            "sending provider request"
        );
        let order: PayPalOrder = self.parse_response(builder.send().await?).await?;
        let url = approval_url(&order)?;

        PaymentSession::new(
            PaymentProvider::PayPal,
            ProviderReference::new(&order.id)?,
            request.reference().clone(),
            map_order_status(&order.status),
            Some(NextAction::RedirectToUrl { url }),
        )
    }

    async fn get_payment_status(
        &self,
        provider_reference: &ProviderReference,
    ) -> Result<PaymentStatusResponse, PaymentError> {
        let token = self.access_token().await?;
        let path = format!("/v2/checkout/orders/{}", provider_reference.as_str());
        tracing::debug!(
            provider = "paypal",
            operation = "get_payment_status",
            "sending provider request"
        );
        let order: PayPalOrder = self
            .parse_response(
                self.client
                    .get(self.endpoint(&path)?)
                    .bearer_auth(token)
                    .send()
                    .await?,
            )
            .await?;

        Ok(PaymentStatusResponse {
            provider: PaymentProvider::PayPal,
            provider_reference: ProviderReference::new(order.id)?,
            status: map_order_status(&order.status),
        })
    }

    async fn refund_payment(
        &self,
        _request: RefundRequest,
    ) -> Result<RefundResponse, PaymentError> {
        Err(PaymentError::UnsupportedOperation(
            "paypal refunds are not implemented in v1".to_owned(),
        ))
    }

    async fn parse_webhook(
        &self,
        request: WebhookRequest<'_>,
    ) -> Result<PaymentEvent, PaymentError> {
        tracing::debug!(
            provider = "paypal",
            operation = "parse_webhook",
            payload_len = request.payload.len(),
            "verifying webhook signature"
        );
        self.verify_webhook_signature(&request).await?;
        parse_event(request.payload)
    }
}

#[async_trait]
impl CapturablePaymentConnector for PayPalConnector {
    async fn capture_payment(
        &self,
        request: CaptureRequest,
    ) -> Result<CaptureResponse, PaymentError> {
        let token = self.access_token().await?;
        let path = format!(
            "/v2/checkout/orders/{}/capture",
            request.provider_reference.as_str()
        );
        tracing::debug!(
            provider = "paypal",
            operation = "capture_payment",
            "sending provider request"
        );
        let order: PayPalOrder = self
            .parse_response(
                self.client
                    .post(self.endpoint(&path)?)
                    .bearer_auth(token)
                    .header("PayPal-Request-Id", request.idempotency_key.as_str())
                    .json(&serde_json::json!({}))
                    .send()
                    .await?,
            )
            .await?;

        Ok(CaptureResponse {
            provider: PaymentProvider::PayPal,
            provider_reference: ProviderReference::new(order.id)?,
            status: map_order_status(&order.status),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{Money, PaymentMethod};
    use secrecy::SecretString;
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_string_contains, header, method, path},
    };

    use super::*;

    #[tokio::test]
    async fn create_order_fetches_token_and_returns_approval_url() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/oauth2/token"))
            .and(body_string_contains("grant_type=client_credentials"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "token_test",
                "expires_in": 3600
            })))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v2/checkout/orders"))
            .and(body_string_contains("\"reference_id\":\"ORDER-1\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "PAYPAL-ORDER-1",
                "status": "CREATED",
                "links": [{
                    "href": "https://paypal.example/approve",
                    "rel": "approve"
                }]
            })))
            .mount(&server)
            .await;
        let config = PayPalConfig::sandbox(
            SecretString::from("client_id".to_owned()),
            SecretString::from("client_secret".to_owned()),
        )
        .expect("config should be valid")
        .base_url(Url::parse(&server.uri()).expect("mock url should parse"));
        let connector = PayPalConnector::new(config).expect("connector should build");
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(1_000, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::paypal())
            .return_url("https://example.com/return")
            .expect("return url should be valid")
            .cancel_url("https://example.com/cancel")
            .expect("cancel url should be valid")
            .build()
            .expect("request should be valid");

        let session = connector
            .create_payment(request)
            .await
            .expect("session should be created");

        assert_eq!(session.provider_reference.as_str(), "PAYPAL-ORDER-1");
        assert!(matches!(
            session.next_action,
            Some(NextAction::RedirectToUrl { .. })
        ));
    }

    #[tokio::test]
    async fn status_capture_and_webhook_are_normalized() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/oauth2/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "token_test",
                "expires_in": 3600
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/v2/checkout/orders/PAYPAL-ORDER-1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "PAYPAL-ORDER-1",
                "status": "APPROVED",
                "links": []
            })))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v2/checkout/orders/PAYPAL-ORDER-1/capture"))
            .and(header("paypal-request-id", "ORDER-1:capture"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "PAYPAL-ORDER-1",
                "status": "COMPLETED",
                "links": []
            })))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v1/notifications/verify-webhook-signature"))
            .and(body_string_contains("\"webhook_id\":\"WH-CONFIGURED\""))
            .and(body_string_contains(
                "\"transmission_id\":\"transmission-1\"",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "verification_status": "SUCCESS"
            })))
            .mount(&server)
            .await;
        let config = PayPalConfig::sandbox(
            SecretString::from("client_id".to_owned()),
            SecretString::from("client_secret".to_owned()),
        )
        .expect("config should be valid")
        .webhook_id("WH-CONFIGURED")
        .expect("webhook id should be valid")
        .base_url(Url::parse(&server.uri()).expect("mock url should parse"));
        let connector = PayPalConnector::new(config).expect("connector should build");
        let reference =
            ProviderReference::new("PAYPAL-ORDER-1").expect("reference should be valid");

        let status = connector
            .get_payment_status(&reference)
            .await
            .expect("status should parse");
        let capture = connector
            .capture_payment(CaptureRequest {
                provider: PaymentProvider::PayPal,
                provider_reference: reference.clone(),
                idempotency_key: crate::IdempotencyKey::new("ORDER-1:capture")
                    .expect("key should be valid"),
            })
            .await
            .expect("capture should parse");
        let mut headers = http::HeaderMap::new();
        headers.insert("paypal-auth-algo", "SHA256withRSA".parse().expect("header"));
        headers.insert(
            "paypal-cert-url",
            "https://paypal.example/cert".parse().expect("header"),
        );
        headers.insert(
            "paypal-transmission-id",
            "transmission-1".parse().expect("header"),
        );
        headers.insert(
            "paypal-transmission-sig",
            "signature".parse().expect("header"),
        );
        headers.insert(
            "paypal-transmission-time",
            "2026-04-26T00:00:00Z".parse().expect("header"),
        );
        let payload = br#"{
            "id":"WH-1",
            "event_type":"CHECKOUT.ORDER.COMPLETED",
            "resource":{"id":"PAYPAL-ORDER-1"}
        }"#;
        let event = connector
            .parse_webhook(WebhookRequest::new(payload, headers))
            .await
            .expect("webhook should parse");

        assert_eq!(status.status, crate::PaymentStatus::Authorized);
        assert_eq!(capture.status, crate::PaymentStatus::Succeeded);
        assert_eq!(event.provider_reference.as_str(), "PAYPAL-ORDER-1");
    }
}
