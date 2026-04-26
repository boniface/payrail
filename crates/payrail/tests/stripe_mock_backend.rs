#![cfg(feature = "stripe")]

use hmac::{Hmac, KeyInit, Mac};
use payrail::{
    IdempotencyKey, Money, PaymentConnector, PaymentError, PaymentMethod, PaymentProvider,
    RefundRequest, WebhookRequest,
};
use payrail::{StripeConfig, StripeConnector};
use secrecy::SecretString;
use serde_json::json;
use sha2::Sha256;
use url::Url;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_string_contains, header, method, path},
};

#[tokio::test]
async fn stripe_mock_backend_covers_payment_status_refund_and_webhook() {
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
    let connector = stripe_connector(&server, Some("whsec_test"));
    let create = payrail::CreatePaymentRequest::builder()
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
        .create_payment(create)
        .await
        .expect("session should be created");
    let status = connector
        .get_payment_status(&session.provider_reference)
        .await
        .expect("status should parse");
    let refund = connector
        .refund_payment(RefundRequest {
            provider: PaymentProvider::Stripe,
            provider_reference: session.provider_reference,
            idempotency_key: IdempotencyKey::new("ORDER-1:refund").expect("key should be valid"),
            amount: None,
            reason: None,
        })
        .await
        .expect("refund should parse");
    let event = connector
        .parse_webhook(stripe_webhook_request(
            br#"{
                "id":"evt_123",
                "type":"payment_intent.succeeded",
                "data":{"object":{"id":"pi_123","client_reference_id":"ORDER-1"}}
            }"#,
            "whsec_test",
        ))
        .await
        .expect("webhook should parse");

    assert_eq!(status.status, payrail::PaymentStatus::Succeeded);
    assert_eq!(refund.provider_reference.as_str(), "re_123");
    assert_eq!(event.provider_reference.as_str(), "pi_123");
}

#[tokio::test]
async fn stripe_stablecoin_checkout_requires_usd() {
    let server = MockServer::start().await;
    let connector = stripe_connector(&server, None);
    let request = payrail::CreatePaymentRequest::builder()
        .amount(Money::new_minor(1_000, "EUR").expect("money should be valid"))
        .reference("ORDER-2")
        .expect("reference should be valid")
        .payment_method(PaymentMethod::stablecoin_usdc())
        .return_url("https://example.com/success")
        .expect("return url should be valid")
        .cancel_url("https://example.com/cancel")
        .expect("cancel url should be valid")
        .build()
        .expect("request should be valid");

    assert!(matches!(
        connector.create_payment(request).await,
        Err(PaymentError::UnsupportedCurrency(_))
    ));
}

#[tokio::test]
async fn stripe_stablecoin_checkout_rejects_non_usdc_assets() {
    let server = MockServer::start().await;
    let connector = stripe_connector(&server, None);
    let request = payrail::CreatePaymentRequest::builder()
        .amount(Money::new_minor(1_000, "USD").expect("money should be valid"))
        .reference("ORDER-2")
        .expect("reference should be valid")
        .payment_method(PaymentMethod::stablecoin_usdt())
        .return_url("https://example.com/success")
        .expect("return url should be valid")
        .cancel_url("https://example.com/cancel")
        .expect("cancel url should be valid")
        .build()
        .expect("request should be valid");

    assert!(matches!(
        connector.create_payment(request).await,
        Err(PaymentError::UnsupportedPaymentMethod(message))
            if message == "stripe stablecoin checkout currently supports USDC only"
    ));
}

fn stripe_connector(server: &MockServer, webhook_secret: Option<&str>) -> StripeConnector {
    let config = StripeConfig::new(SecretString::from("sk_test_payrail".to_owned()))
        .expect("config should be valid")
        .webhook_secret(webhook_secret.map(|secret| SecretString::from(secret.to_owned())))
        .api_base(Url::parse(&server.uri()).expect("mock url should parse"));
    StripeConnector::new(config).expect("connector should build")
}

fn stripe_webhook_request(payload: &'static [u8], secret: &str) -> WebhookRequest<'static> {
    let timestamp = time::OffsetDateTime::now_utc().unix_timestamp().to_string();
    let mut signed_payload = Vec::with_capacity(timestamp.len() + 1 + payload.len());
    signed_payload.extend_from_slice(timestamp.as_bytes());
    signed_payload.push(b'.');
    signed_payload.extend_from_slice(payload);
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("hmac should initialize");
    mac.update(&signed_payload);
    let signature = format!(
        "t={timestamp},v1={}",
        mac.finalize()
            .into_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
    let mut headers = http::HeaderMap::new();
    headers.insert(
        "stripe-signature",
        signature.parse().expect("signature header should parse"),
    );
    WebhookRequest::new(payload, headers)
}
