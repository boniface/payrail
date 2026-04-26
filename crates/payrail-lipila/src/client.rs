use async_trait::async_trait;
use payrail_core::{
    CreatePaymentRequest, CurrencyCode, NextAction, PaymentConnector, PaymentError, PaymentEvent,
    PaymentMethod, PaymentProvider, PaymentSession, PaymentStatusResponse, ProviderErrorDetails,
    ProviderReference, RefundRequest, RefundResponse, WebhookRequest,
};
use secrecy::ExposeSecret;
use url::Url;

use crate::{
    callback::parse_callback,
    collection::collection_request,
    config::LipilaConfig,
    mapper::{map_payment_type, map_status},
    models::LipilaCollectionResponse,
    webhook::verify_signature,
};

/// Lipila PayRail connector.
#[derive(Debug, Clone)]
pub struct LipilaConnector {
    config: LipilaConfig,
    client: reqwest::Client,
}

impl LipilaConnector {
    /// Creates a Lipila connector.
    ///
    /// # Errors
    ///
    /// Returns an error when the HTTP client cannot be built.
    pub fn new(config: LipilaConfig) -> Result<Self, PaymentError> {
        let client = reqwest::Client::builder()
            .user_agent("payrail-rs/0.1 (+https://github.com/boniface/payrail)")
            .timeout(config.request_timeout_value())
            .build()?;
        Ok(Self { config, client })
    }

    fn endpoint(&self, path: &str) -> Result<Url, PaymentError> {
        self.config
            .base_url_value()
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

        Err(PaymentError::ProviderDetails {
            details: ProviderErrorDetails {
                provider: PaymentProvider::Lipila,
                status: status.as_u16(),
                code: None,
                request_id: None,
                message: "lipila request failed".to_owned(),
            },
        })
    }
}

#[async_trait]
impl PaymentConnector for LipilaConnector {
    fn provider(&self) -> PaymentProvider {
        PaymentProvider::Lipila
    }

    async fn create_payment(
        &self,
        request: CreatePaymentRequest,
    ) -> Result<PaymentSession, PaymentError> {
        if !matches!(request.payment_method(), PaymentMethod::MobileMoney(_)) {
            return Err(PaymentError::UnsupportedPaymentMethod(
                "lipila only supports mobile money routes".to_owned(),
            ));
        }
        let body = collection_request(&request)?;
        let mut builder = self
            .client
            .post(self.endpoint("/api/v1/collections/mobile-money")?)
            .header("x-api-key", self.config.api_key().expose_secret())
            .json(&body);
        if let Some(callback_url) = request.callback_url() {
            builder = builder.header("callbackUrl", callback_url.as_str());
        }

        tracing::debug!(
            provider = "lipila",
            operation = "create_payment",
            has_callback_url = request.callback_url().is_some(),
            "sending provider request"
        );
        let response: LipilaCollectionResponse = self.parse_response(builder.send().await?).await?;
        let _normalized_amount = response.amount.as_i64().map(|amount| {
            CurrencyCode::new(&response.currency)
                .and_then(|currency| currency.major_integer_to_minor_units(amount))
                .and_then(|minor| payrail_core::Money::new_minor(minor, &response.currency))
        });
        let _redacted_account_number = response.account_number.as_str();
        let _operator = response.payment_type.as_deref().map(map_payment_type);
        let provider_reference = response
            .external_id
            .as_deref()
            .or(response.identifier.as_deref())
            .unwrap_or(response.reference_id.as_str());
        let message = response
            .message
            .unwrap_or_else(|| "complete the mobile money prompt".to_owned());

        PaymentSession::new(
            PaymentProvider::Lipila,
            ProviderReference::new(provider_reference)?,
            request.reference().clone(),
            map_status(&response.status),
            Some(NextAction::MobileMoneyPrompt { message }),
        )
    }

    async fn get_payment_status(
        &self,
        provider_reference: &ProviderReference,
    ) -> Result<PaymentStatusResponse, PaymentError> {
        let mut url = self.endpoint("/api/v1/collections/check-status")?;
        url.query_pairs_mut()
            .append_pair("referenceId", provider_reference.as_str());
        tracing::debug!(
            provider = "lipila",
            operation = "get_payment_status",
            "sending provider request"
        );
        let response: LipilaCollectionResponse = self
            .parse_response(
                self.client
                    .get(url)
                    .header("x-api-key", self.config.api_key().expose_secret())
                    .send()
                    .await?,
            )
            .await?;

        Ok(PaymentStatusResponse {
            provider: PaymentProvider::Lipila,
            provider_reference: ProviderReference::new(
                response
                    .external_id
                    .as_deref()
                    .or(response.identifier.as_deref())
                    .unwrap_or(response.reference_id.as_str()),
            )?,
            status: map_status(&response.status),
        })
    }

    async fn refund_payment(
        &self,
        _request: RefundRequest,
    ) -> Result<RefundResponse, PaymentError> {
        Err(PaymentError::UnsupportedOperation(
            "lipila refunds are not implemented in v1".to_owned(),
        ))
    }

    async fn parse_webhook(
        &self,
        request: WebhookRequest<'_>,
    ) -> Result<PaymentEvent, PaymentError> {
        let secret = self.config.webhook_secret_value().ok_or_else(|| {
            PaymentError::InvalidConfiguration("missing lipila webhook secret".to_owned())
        })?;
        let webhook_id = header(&request, "webhook-id")?;
        let timestamp = header(&request, "webhook-timestamp")?;
        let signature = header(&request, "webhook-signature")?;
        tracing::debug!(
            provider = "lipila",
            operation = "parse_webhook",
            payload_len = request.payload.len(),
            "verifying webhook signature"
        );
        verify_signature(webhook_id, timestamp, signature, request.payload, secret)?;
        parse_callback(request.payload)
    }
}

fn header<'a>(request: &'a WebhookRequest<'_>, name: &str) -> Result<&'a str, PaymentError> {
    request
        .headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .ok_or(PaymentError::WebhookVerificationFailed)
}

#[cfg(test)]
mod tests {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use hmac::{Hmac, KeyInit, Mac};
    use payrail_core::{Money, PaymentMethod};
    use secrecy::SecretString;
    use serde_json::json;
    use sha2::Sha256;
    use time::OffsetDateTime;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_string_contains, header, method, path},
    };

    use super::*;

    #[tokio::test]
    async fn create_collection_sends_required_headers_and_body() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/collections/mobile-money"))
            .and(header("x-api-key", "lipila_test_key"))
            .and(header("callbackurl", "https://example.com/webhook"))
            .and(body_string_contains("\"referenceId\":\"ORDER-1\""))
            .and(body_string_contains("\"accountNumber\":\"260971234567\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "referenceId": "ORDER-1",
                "currency": "ZMW",
                "amount": 50,
                "accountNumber": "260971234567",
                "status": "Pending",
                "paymentType": "MTNMoney",
                "externalId": "LIPILA-1",
                "identifier": "IDENT-1",
                "message": "Prompt sent"
            })))
            .mount(&server)
            .await;
        let config = LipilaConfig::sandbox(SecretString::from("lipila_test_key".to_owned()))
            .expect("config should be valid")
            .base_url(Url::parse(&server.uri()).expect("mock url should parse"));
        let connector = LipilaConnector::new(config).expect("connector should build");
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(5_000, "ZMW").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(
                PaymentMethod::mobile_money_zambia("260971234567").expect("method should be valid"),
            )
            .callback_url("https://example.com/webhook")
            .expect("callback url should be valid")
            .build()
            .expect("request should be valid");

        let session = connector
            .create_payment(request)
            .await
            .expect("session should be created");

        assert_eq!(session.provider_reference.as_str(), "LIPILA-1");
        assert!(matches!(
            session.next_action,
            Some(NextAction::MobileMoneyPrompt { .. })
        ));
    }

    #[tokio::test]
    async fn status_and_webhook_are_normalized() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/collections/check-status"))
            .and(header("x-api-key", "lipila_test_key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "referenceId": "ORDER-1",
                "currency": "ZMW",
                "amount": 50,
                "accountNumber": "260971234567",
                "status": "Successful",
                "paymentType": "MTNMoney",
                "externalId": "LIPILA-1",
                "identifier": "IDENT-1",
                "message": "Paid"
            })))
            .mount(&server)
            .await;
        let raw_secret = b"lipila-webhook-secret";
        let config = LipilaConfig::sandbox(SecretString::from("lipila_test_key".to_owned()))
            .expect("config should be valid")
            .webhook_secret(Some(SecretString::from(STANDARD.encode(raw_secret))))
            .base_url(Url::parse(&server.uri()).expect("mock url should parse"));
        let connector = LipilaConnector::new(config).expect("connector should build");
        let reference = ProviderReference::new("ORDER-1").expect("reference should be valid");
        let status = connector
            .get_payment_status(&reference)
            .await
            .expect("status should parse");
        let payload = br#"{
            "referenceId":"ORDER-1",
            "currency":"ZMW",
            "amount":50,
            "accountNumber":"260971234567",
            "status":"Successful",
            "paymentType":"MTNMoney",
            "type":"collection",
            "ipAddress":"127.0.0.1",
            "identifier":"IDENT-1",
            "message":"Paid",
            "externalId":"LIPILA-1"
        }"#;
        let timestamp = OffsetDateTime::now_utc().unix_timestamp().to_string();
        let signed = format!("evt_1.{timestamp}.{}", String::from_utf8_lossy(payload));
        let mut mac = Hmac::<Sha256>::new_from_slice(raw_secret).expect("hmac should initialize");
        mac.update(signed.as_bytes());
        let signature = format!("v1,{}", STANDARD.encode(mac.finalize().into_bytes()));
        let mut headers = http::HeaderMap::new();
        headers.insert("webhook-id", "evt_1".parse().expect("id should parse"));
        headers.insert(
            "webhook-timestamp",
            timestamp.parse().expect("timestamp should parse"),
        );
        headers.insert(
            "webhook-signature",
            signature.parse().expect("signature should parse"),
        );
        let event = connector
            .parse_webhook(WebhookRequest::new(payload, headers))
            .await
            .expect("webhook should parse");

        assert_eq!(status.status, payrail_core::PaymentStatus::Succeeded);
        assert_eq!(event.provider_reference.as_str(), "LIPILA-1");
    }
}
