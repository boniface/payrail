use std::fmt::Write as _;

use crate::PaymentError;
use hmac::{Hmac, KeyInit, Mac};
use secrecy::{ExposeSecret, SecretString};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use time::OffsetDateTime;

type HmacSha256 = Hmac<Sha256>;
const SIGNATURE_TOLERANCE_SECONDS: i64 = 300;

pub(super) fn verify_signature(
    payload: &[u8],
    signature_header: &str,
    secret: &SecretString,
) -> Result<(), PaymentError> {
    let timestamp =
        signature_part(signature_header, "t").ok_or(PaymentError::WebhookVerificationFailed)?;
    reject_stale_timestamp(timestamp)?;
    let expected_signatures = signature_values(signature_header, "v1");
    let mut signed_payload = Vec::with_capacity(timestamp.len() + 1 + payload.len());
    signed_payload.extend_from_slice(timestamp.as_bytes());
    signed_payload.push(b'.');
    signed_payload.extend_from_slice(payload);

    let mut mac = HmacSha256::new_from_slice(secret.expose_secret().as_bytes())
        .map_err(|_| PaymentError::WebhookVerificationFailed)?;
    mac.update(&signed_payload);
    let actual = hex_encode(&mac.finalize().into_bytes());

    if expected_signatures
        .into_iter()
        .any(|expected| actual.as_bytes().ct_eq(expected.as_bytes()).into())
    {
        return Ok(());
    }

    Err(PaymentError::WebhookVerificationFailed)
}

fn reject_stale_timestamp(timestamp: &str) -> Result<(), PaymentError> {
    let timestamp = timestamp
        .parse::<i64>()
        .map_err(|_| PaymentError::WebhookVerificationFailed)?;
    let timestamp = OffsetDateTime::from_unix_timestamp(timestamp)
        .map_err(|_| PaymentError::WebhookVerificationFailed)?;
    let age = OffsetDateTime::now_utc() - timestamp;
    if age.whole_seconds().abs() > SIGNATURE_TOLERANCE_SECONDS {
        return Err(PaymentError::WebhookVerificationFailed);
    }

    Ok(())
}

fn signature_part<'a>(header: &'a str, key: &str) -> Option<&'a str> {
    header.split(',').find_map(|part| {
        let (part_key, value) = part.split_once('=')?;
        (part_key == key).then_some(value)
    })
}

fn signature_values<'a>(header: &'a str, key: &str) -> Vec<&'a str> {
    header
        .split(',')
        .filter_map(|part| {
            let (part_key, value) = part.split_once('=')?;
            (part_key == key).then_some(value)
        })
        .collect()
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bad_signature() {
        let secret = SecretString::from("whsec_test".to_owned());

        assert!(matches!(
            verify_signature(b"{}", "t=1,v1=bad", &secret),
            Err(PaymentError::WebhookVerificationFailed)
        ));
    }

    #[test]
    fn accepts_valid_signature() {
        let secret = SecretString::from("whsec_test".to_owned());
        let payload = b"{}";
        let timestamp = OffsetDateTime::now_utc().unix_timestamp().to_string();
        let mut signed_payload = Vec::new();
        signed_payload.extend_from_slice(timestamp.as_bytes());
        signed_payload.push(b'.');
        signed_payload.extend_from_slice(payload);
        let mut mac = HmacSha256::new_from_slice(secret.expose_secret().as_bytes())
            .expect("hmac should initialize");
        mac.update(&signed_payload);
        let signature = format!(
            "t={timestamp},v1={}",
            hex_encode(&mac.finalize().into_bytes())
        );

        verify_signature(payload, &signature, &secret).expect("signature should verify");
    }

    #[test]
    fn accepts_any_matching_v1_signature() {
        let secret = SecretString::from("whsec_test".to_owned());
        let payload = b"{}";
        let timestamp = OffsetDateTime::now_utc().unix_timestamp().to_string();
        let mut signed_payload = Vec::new();
        signed_payload.extend_from_slice(timestamp.as_bytes());
        signed_payload.push(b'.');
        signed_payload.extend_from_slice(payload);
        let mut mac = HmacSha256::new_from_slice(secret.expose_secret().as_bytes())
            .expect("hmac should initialize");
        mac.update(&signed_payload);
        let signature = format!(
            "t={timestamp},v1=bad,v1={}",
            hex_encode(&mac.finalize().into_bytes())
        );

        verify_signature(payload, &signature, &secret).expect("signature should verify");
    }

    #[test]
    fn rejects_stale_signature() {
        let secret = SecretString::from("whsec_test".to_owned());
        let payload = b"{}";
        let timestamp = "1";
        let mut signed_payload = Vec::new();
        signed_payload.extend_from_slice(timestamp.as_bytes());
        signed_payload.push(b'.');
        signed_payload.extend_from_slice(payload);
        let mut mac = HmacSha256::new_from_slice(secret.expose_secret().as_bytes())
            .expect("hmac should initialize");
        mac.update(&signed_payload);
        let signature = format!(
            "t={timestamp},v1={}",
            hex_encode(&mac.finalize().into_bytes())
        );

        assert!(matches!(
            verify_signature(payload, &signature, &secret),
            Err(PaymentError::WebhookVerificationFailed)
        ));
    }
}
