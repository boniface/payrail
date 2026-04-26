#![cfg(feature = "paypal")]

use payrail::{
    CapturablePaymentConnector, CaptureRequest, IdempotencyKey, Money, PaymentConnector,
    PaymentMethod, PaymentProvider, ProviderReference, RefundRequest, WebhookRequest,
};
use payrail::{PayPalConfig, PayPalConnector};
use secrecy::SecretString;
use serde_json::json;
use url::Url;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_string_contains, header, method, path},
};

#[tokio::test]
async fn paypal_mock_backend_covers_order_status_capture_refund_and_webhook() {
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
        .and(header("paypal-request-id", "ORDER-1:create"))
        .and(body_string_contains("\"reference_id\":\"ORDER-1\""))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "PAYPAL-ORDER-1",
            "status": "CREATED",
            "links": [{"href": "https://paypal.example/approve", "rel": "approve"}]
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
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "verification_status": "SUCCESS"
        })))
        .mount(&server)
        .await;
    let connector = paypal_connector(&server);
    let create = payrail::CreatePaymentRequest::builder()
        .amount(Money::new_minor(1_000, "USD").expect("money should be valid"))
        .reference("ORDER-1")
        .expect("reference should be valid")
        .payment_method(PaymentMethod::paypal())
        .return_url("https://example.com/return")
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
        .expect("order should be created");
    let reference = ProviderReference::new(session.provider_reference.as_str())
        .expect("reference should be valid");
    let status = connector
        .get_payment_status(&reference)
        .await
        .expect("status should parse");
    let capture = connector
        .capture_payment(CaptureRequest {
            provider: PaymentProvider::PayPal,
            provider_reference: reference.clone(),
            idempotency_key: IdempotencyKey::new("ORDER-1:capture").expect("key should be valid"),
        })
        .await
        .expect("capture should parse");
    let refund = connector
        .refund_payment(RefundRequest {
            provider: PaymentProvider::PayPal,
            provider_reference: reference,
            idempotency_key: IdempotencyKey::new("ORDER-1:refund").expect("key should be valid"),
            amount: None,
            reason: None,
        })
        .await;
    let event = connector
        .parse_webhook(paypal_webhook_request())
        .await
        .expect("webhook should parse");

    assert_eq!(status.status, payrail::PaymentStatus::Authorized);
    assert_eq!(capture.status, payrail::PaymentStatus::Succeeded);
    assert!(matches!(
        refund,
        Err(payrail::PaymentError::UnsupportedOperation(_))
    ));
    assert_eq!(event.provider_reference.as_str(), "PAYPAL-ORDER-1");
}

fn paypal_connector(server: &MockServer) -> PayPalConnector {
    let config = PayPalConfig::sandbox(
        SecretString::from("client_id".to_owned()),
        SecretString::from("client_secret".to_owned()),
    )
    .expect("config should be valid")
    .webhook_id("WH-CONFIGURED")
    .expect("webhook id should be valid")
    .base_url(Url::parse(&server.uri()).expect("mock url should parse"));
    PayPalConnector::new(config).expect("connector should build")
}

fn paypal_webhook_request() -> WebhookRequest<'static> {
    let payload = br#"{
        "id":"WH-1",
        "event_type":"CHECKOUT.ORDER.COMPLETED",
        "resource":{"id":"PAYPAL-ORDER-1"}
    }"#;
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
    WebhookRequest::new(payload, headers)
}
