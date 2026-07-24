/// Normalized provider field.
pub const TELEMETRY_FIELD_PROVIDER: &str = "payrail.provider";
/// Normalized operation field.
pub const TELEMETRY_FIELD_OPERATION: &str = "payrail.operation";
/// Normalized payment method category field.
pub const TELEMETRY_FIELD_PAYMENT_METHOD: &str = "payrail.payment_method";
/// Checkout UI mode field.
pub const TELEMETRY_FIELD_CHECKOUT_UI_MODE: &str = "payrail.checkout_ui_mode";
/// Normalized status field.
pub const TELEMETRY_FIELD_STATUS: &str = "payrail.status";
/// Normalized event type field.
pub const TELEMETRY_FIELD_EVENT_TYPE: &str = "payrail.event_type";
/// Boolean idempotency-key presence field.
pub const TELEMETRY_FIELD_HAS_IDEMPOTENCY_KEY: &str = "payrail.has_idempotency_key";
/// Boolean callback-url presence field.
pub const TELEMETRY_FIELD_HAS_CALLBACK_URL: &str = "payrail.has_callback_url";
/// Webhook payload byte length field.
pub const TELEMETRY_FIELD_PAYLOAD_LEN: &str = "payrail.payload_len";
/// Provider HTTP status field.
pub const TELEMETRY_FIELD_HTTP_STATUS: &str = "payrail.http_status";
/// Low-cardinality operation result field.
pub const TELEMETRY_FIELD_RESULT: &str = "payrail.result";
/// Normalized error kind field.
pub const TELEMETRY_FIELD_ERROR_KIND: &str = "payrail.error_kind";
/// Normalized fraud risk decision field.
#[cfg(feature = "fraud")]
pub const TELEMETRY_FIELD_RISK_DECISION: &str = "payrail.risk_decision";
/// Normalized fraud risk level field.
#[cfg(feature = "fraud")]
pub const TELEMETRY_FIELD_RISK_LEVEL: &str = "payrail.risk_level";
/// Fraud reason count field.
#[cfg(feature = "fraud")]
pub const TELEMETRY_FIELD_REASON_COUNT: &str = "payrail.reason_count";
/// Fraud policy mode field.
#[cfg(feature = "fraud")]
pub const TELEMETRY_FIELD_POLICY_MODE: &str = "payrail.policy_mode";
/// Provider I/O outcome field.
#[cfg(feature = "fraud")]
pub const TELEMETRY_FIELD_PROVIDER_IO: &str = "payrail.provider_io";
/// Normalized fraud event type field.
#[cfg(feature = "fraud")]
pub const TELEMETRY_FIELD_FRAUD_EVENT_TYPE: &str = "payrail.fraud_event_type";
