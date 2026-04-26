#![no_main]

use futures::executor::block_on;
use hmac::{Hmac, KeyInit, Mac};
use http::HeaderMap;
use libfuzzer_sys::fuzz_target;
use payrail::{PaymentConnector, StripeConfig, StripeConnector, WebhookRequest};
use secrecy::{ExposeSecret, SecretString};
use sha2::Sha256;
use time::OffsetDateTime;

type HmacSha256 = Hmac<Sha256>;

fuzz_target!(|payload: &[u8]| {
    let secret = SecretString::from("whsec_fuzz".to_owned());
    let timestamp = OffsetDateTime::now_utc().unix_timestamp().to_string();
    let mut signed_payload = Vec::with_capacity(timestamp.len() + 1 + payload.len());
    signed_payload.extend_from_slice(timestamp.as_bytes());
    signed_payload.push(b'.');
    signed_payload.extend_from_slice(payload);
    let mut mac = HmacSha256::new_from_slice(secret.expose_secret().as_bytes())
        .expect("hmac should initialize for fuzz secret");
    mac.update(&signed_payload);
    let signature = format!("t={timestamp},v1={}", hex_encode(&mac.finalize().into_bytes()));
    let mut headers = HeaderMap::new();
    if let Ok(value) = signature.parse() {
        headers.insert("stripe-signature", value);
    }
    let config = StripeConfig::new(SecretString::from("sk_test_fuzz".to_owned()))
        .expect("stripe config should be valid")
        .webhook_secret(Some(secret));
    let connector = StripeConnector::new(config).expect("stripe connector should build");

    let _ = block_on(connector.parse_webhook(WebhookRequest::new(payload, headers)));
});

fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut output = String::with_capacity(bytes.len() * 2);
    bytes.iter().for_each(|byte| {
        let _ = write!(output, "{byte:02x}");
    });
    output
}
