#![cfg(feature = "lipila")]

use base64::{Engine as _, engine::general_purpose::STANDARD};
use hmac::{Hmac, KeyInit, Mac};
use payrail::{
    IdempotencyKey, Money, PaymentMethod, PaymentProvider, ProviderReference, RefundRequest,
    WebhookRequest,
};
use payrail::{LipilaConfig, LipilaConnector};
use secrecy::SecretString;
use serde_json::json;
use sha2::Sha256;
use time::OffsetDateTime;
use url::Url;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_string_contains, header, method, path},
};

#[tokio::test]
async fn lipila_mock_backend_covers_collection_status_refund_and_webhook() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/collections/mobile-money"))
        .and(header("x-api-key", "lipila_test_key"))
        .and(header("callbackurl", "https://example.com/webhook"))
        .and(body_string_contains("\"referenceId\":\"ORDER-1\""))
        .respond_with(ResponseTemplate::new(200).set_body_json(collection_response("Pending")))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/collections/check-status"))
        .and(header("x-api-key", "lipila_test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(collection_response("Successful")))
        .mount(&server)
        .await;
    let secret = b"lipila-webhook-secret";
    let connector = lipila_connector(&server, secret);
    let create = payrail::CreatePaymentRequest::builder()
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
        .create_payment(create)
        .await
        .expect("collection should be created");
    let status = connector
        .get_payment_status(&ProviderReference::new("ORDER-1").expect("reference should be valid"))
        .await
        .expect("status should parse");
    let refund = connector
        .refund_payment(RefundRequest {
            provider: PaymentProvider::Lipila,
            provider_reference: session.provider_reference,
            idempotency_key: IdempotencyKey::new("ORDER-1:refund").expect("key should be valid"),
            amount: None,
            reason: None,
        })
        .await;
    let event = connector
        .parse_webhook(lipila_webhook_request(secret))
        .await
        .expect("webhook should parse");

    assert_eq!(status.status, payrail::PaymentStatus::Succeeded);
    assert!(matches!(
        refund,
        Err(payrail::PaymentError::UnsupportedOperation(_))
    ));
    assert_eq!(event.provider_reference.as_str(), "LIPILA-1");
}

fn collection_response(status: &str) -> serde_json::Value {
    json!({
        "referenceId": "ORDER-1",
        "currency": "ZMW",
        "amount": 50,
        "accountNumber": "260971234567",
        "status": status,
        "paymentType": "MTNMoney",
        "externalId": "LIPILA-1",
        "identifier": "IDENT-1",
        "message": "Prompt sent"
    })
}

fn lipila_connector(server: &MockServer, raw_secret: &[u8]) -> LipilaConnector {
    let config = LipilaConfig::sandbox(SecretString::from("lipila_test_key".to_owned()))
        .expect("config should be valid")
        .webhook_secret(Some(SecretString::from(STANDARD.encode(raw_secret))))
        .base_url(Url::parse(&server.uri()).expect("mock url should parse"));
    LipilaConnector::new(config).expect("connector should build")
}

fn lipila_webhook_request(raw_secret: &'static [u8]) -> WebhookRequest<'static> {
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
    WebhookRequest::new(payload, headers)
}
