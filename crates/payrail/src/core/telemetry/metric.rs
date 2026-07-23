/// Payment request counter metric.
pub const TELEMETRY_METRIC_PAYMENT_REQUESTS_TOTAL: &str = "payrail_payment_requests_total";
/// Provider request counter metric.
pub const TELEMETRY_METRIC_PROVIDER_REQUESTS_TOTAL: &str = "payrail_provider_requests_total";
/// Provider request duration histogram metric.
pub const TELEMETRY_METRIC_PROVIDER_REQUEST_DURATION_MS: &str =
    "payrail_provider_request_duration_ms";
/// Webhook counter metric.
pub const TELEMETRY_METRIC_WEBHOOKS_TOTAL: &str = "payrail_webhooks_total";
/// Fraud assessment counter metric.
#[cfg(feature = "fraud")]
pub const TELEMETRY_METRIC_FRAUD_ASSESSMENTS_TOTAL: &str = "payrail_fraud_assessments_total";
/// Fraud policy block counter metric.
#[cfg(feature = "fraud")]
pub const TELEMETRY_METRIC_FRAUD_POLICY_BLOCKS_TOTAL: &str = "payrail_fraud_policy_blocks_total";
