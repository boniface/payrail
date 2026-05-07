#![no_main]

use base64::{Engine as _, engine::general_purpose::STANDARD};
use futures::executor::block_on;
use hmac::{Hmac, KeyInit, Mac};
use http::HeaderMap;
use libfuzzer_sys::fuzz_target;
use payrail::{LipilaConfig, LipilaConnector, WebhookRequest};
use secrecy::SecretString;
use sha2::Sha256;
use time::OffsetDateTime;

type HmacSha256 = Hmac<Sha256>;

fuzz_target!(|payload: &[u8]| {
    let raw_secret = b"lipila-fuzz-secret";
    let timestamp = OffsetDateTime::now_utc().unix_timestamp().to_string();
    let webhook_id = "evt_fuzz";
    let mut signed_payload =
        Vec::with_capacity(webhook_id.len() + timestamp.len() + payload.len() + 2);
    signed_payload.extend_from_slice(webhook_id.as_bytes());
    signed_payload.push(b'.');
    signed_payload.extend_from_slice(timestamp.as_bytes());
    signed_payload.push(b'.');
    signed_payload.extend_from_slice(payload);
    let mut mac = HmacSha256::new_from_slice(raw_secret).expect("hmac should initialize");
    mac.update(&signed_payload);
    let signature = format!("v1,{}", STANDARD.encode(mac.finalize().into_bytes()));
    let mut headers = HeaderMap::new();
    headers.insert("webhook-id", webhook_id.parse().expect("header should parse"));
    headers.insert(
        "webhook-timestamp",
        timestamp.parse().expect("header should parse"),
    );
    if let Ok(value) = signature.parse() {
        headers.insert("webhook-signature", value);
    }
    let config = LipilaConfig::sandbox(SecretString::from("lipila_fuzz_key".to_owned()))
        .expect("lipila config should be valid")
        .webhook_secret(Some(SecretString::from(STANDARD.encode(raw_secret))));
    let connector = LipilaConnector::new(config).expect("lipila connector should build");

    let _ = block_on(connector.parse_webhook(WebhookRequest::new(payload, headers)));
});
