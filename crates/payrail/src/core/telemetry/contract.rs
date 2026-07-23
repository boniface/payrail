use crate::{
    TELEMETRY_FIELD_CHECKOUT_UI_MODE, TELEMETRY_FIELD_ERROR_KIND, TELEMETRY_FIELD_EVENT_TYPE,
    TELEMETRY_FIELD_HAS_CALLBACK_URL, TELEMETRY_FIELD_HAS_IDEMPOTENCY_KEY,
    TELEMETRY_FIELD_HTTP_STATUS, TELEMETRY_FIELD_OPERATION, TELEMETRY_FIELD_PAYLOAD_LEN,
    TELEMETRY_FIELD_PAYMENT_METHOD, TELEMETRY_FIELD_PROVIDER, TELEMETRY_FIELD_RESULT,
    TELEMETRY_FIELD_STATUS,
};
#[cfg(feature = "fraud")]
use crate::{
    TELEMETRY_FIELD_FRAUD_EVENT_TYPE, TELEMETRY_FIELD_POLICY_MODE, TELEMETRY_FIELD_PROVIDER_IO,
    TELEMETRY_FIELD_REASON_COUNT, TELEMETRY_FIELD_RISK_DECISION, TELEMETRY_FIELD_RISK_LEVEL,
};

/// PayRail telemetry contract.
///
/// PayRail emits structured diagnostics through `tracing` when the `telemetry` feature is enabled.
/// Applications own subscriber setup, OpenTelemetry SDK/exporters, sampling, resource attributes,
/// and collector endpoints. PayRail never installs a global subscriber and never configures OTLP.
///
/// Telemetry fields must be low-cardinality and must not contain secrets, customer PII, raw
/// provider payloads, idempotency keys, references, URLs, raw fraud context, or provider response
/// bodies.
pub const TELEMETRY_CONTRACT: &str = "payrail.telemetry.contract.v1";

/// Field names allowed by the telemetry foundation.
pub const ALLOWED_FIELDS: &[&str] = &[
    TELEMETRY_FIELD_PROVIDER,
    TELEMETRY_FIELD_OPERATION,
    TELEMETRY_FIELD_PAYMENT_METHOD,
    TELEMETRY_FIELD_CHECKOUT_UI_MODE,
    TELEMETRY_FIELD_STATUS,
    TELEMETRY_FIELD_EVENT_TYPE,
    TELEMETRY_FIELD_HAS_IDEMPOTENCY_KEY,
    TELEMETRY_FIELD_HAS_CALLBACK_URL,
    TELEMETRY_FIELD_PAYLOAD_LEN,
    TELEMETRY_FIELD_HTTP_STATUS,
    TELEMETRY_FIELD_RESULT,
    TELEMETRY_FIELD_ERROR_KIND,
    #[cfg(feature = "fraud")]
    TELEMETRY_FIELD_RISK_DECISION,
    #[cfg(feature = "fraud")]
    TELEMETRY_FIELD_RISK_LEVEL,
    #[cfg(feature = "fraud")]
    TELEMETRY_FIELD_REASON_COUNT,
    #[cfg(feature = "fraud")]
    TELEMETRY_FIELD_POLICY_MODE,
    #[cfg(feature = "fraud")]
    TELEMETRY_FIELD_PROVIDER_IO,
    #[cfg(feature = "fraud")]
    TELEMETRY_FIELD_FRAUD_EVENT_TYPE,
];

/// Data categories that must not be emitted by PayRail telemetry.
pub const FORBIDDEN_FIELDS: &[&str] = &[
    "api_key",
    "authorization",
    "webhook_secret",
    "idempotency_key",
    "customer_email",
    "customer_phone",
    "raw_webhook_payload",
    "raw_provider_response",
    "device_token",
    "risk_context",
    "merchant_reference",
    "provider_reference",
    "payment_id",
    "full_url",
    "card_data",
    "bank_details",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_lists_allowed_and_forbidden_fields() {
        assert!(ALLOWED_FIELDS.contains(&TELEMETRY_FIELD_PROVIDER));
        assert!(FORBIDDEN_FIELDS.contains(&"idempotency_key"));
        assert!(TELEMETRY_CONTRACT.starts_with("payrail.telemetry"));
    }

    #[test]
    fn allowed_fields_do_not_include_sensitive_categories() {
        for forbidden in FORBIDDEN_FIELDS {
            assert!(
                !ALLOWED_FIELDS.contains(forbidden),
                "forbidden field should not be allowed: {forbidden}"
            );
        }
    }
}
