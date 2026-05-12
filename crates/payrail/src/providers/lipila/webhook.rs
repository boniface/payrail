use crate::PaymentError;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use hmac::{Hmac, KeyInit, Mac};
use secrecy::{ExposeSecret, SecretString};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use time::OffsetDateTime;

type HmacSha256 = Hmac<Sha256>;

pub(super) fn verify_signature(
    id: &str,
    timestamp: &str,
    signatures: &str,
    payload: &[u8],
    secret: &SecretString,
) -> Result<(), PaymentError> {
    reject_stale_timestamp(timestamp)?;
    let secret = STANDARD
        .decode(secret.expose_secret())
        .map_err(|_| PaymentError::WebhookVerificationFailed)?;
    let mut signed_payload = Vec::with_capacity(id.len() + timestamp.len() + payload.len() + 2);
    signed_payload.extend_from_slice(id.as_bytes());
    signed_payload.push(b'.');
    signed_payload.extend_from_slice(timestamp.as_bytes());
    signed_payload.push(b'.');
    signed_payload.extend_from_slice(payload);

    let mut mac =
        HmacSha256::new_from_slice(&secret).map_err(|_| PaymentError::WebhookVerificationFailed)?;
    mac.update(&signed_payload);
    let expected = format!("v1,{}", STANDARD.encode(mac.finalize().into_bytes()));

    if signatures
        .split_whitespace()
        .any(|signature| expected.as_bytes().ct_eq(signature.as_bytes()).into())
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
    if age.whole_seconds().abs() > 300 {
        return Err(PaymentError::WebhookVerificationFailed);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use secrecy::SecretString;
    use time::OffsetDateTime;

    use super::*;

    #[test]
    fn verify_signature_accepts_valid_signature() {
        let raw_secret = b"test-webhook-secret";
        let secret = SecretString::from(STANDARD.encode(raw_secret));
        let timestamp = OffsetDateTime::now_utc().unix_timestamp().to_string();
        let payload = br#"{"referenceId":"ORDER-1"}"#;
        let signed = format!("evt_1.{timestamp}.{}", String::from_utf8_lossy(payload));
        let mut mac = HmacSha256::new_from_slice(raw_secret).expect("hmac should initialize");
        mac.update(signed.as_bytes());
        let signature = format!("v1,{}", STANDARD.encode(mac.finalize().into_bytes()));

        verify_signature("evt_1", &timestamp, &signature, payload, &secret)
            .expect("signature should verify");
    }

    #[test]
    fn verify_signature_rejects_stale_timestamp() {
        let secret = SecretString::from(STANDARD.encode(b"test-webhook-secret"));

        assert!(matches!(
            verify_signature("evt_1", "1", "v1,bad", b"{}", &secret),
            Err(PaymentError::WebhookVerificationFailed)
        ));
    }
}
