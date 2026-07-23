mod contract;
mod emit;
mod field;
mod metric;
mod operation;
#[cfg(feature = "otel")]
mod otel;
mod value;

pub use contract::{ALLOWED_FIELDS, FORBIDDEN_FIELDS, TELEMETRY_CONTRACT};
#[cfg(feature = "fraud")]
pub(crate) use emit::emit_fraud_assessment;
#[cfg(any(feature = "lipila", feature = "paypal", feature = "stripe"))]
pub(crate) use emit::emit_provider_request_result;
pub(crate) use emit::emit_result;
pub use field::*;
pub use metric::*;
pub use operation::TelemetryOperation;
#[cfg(feature = "otel")]
pub use otel::{PayRailOtelMetrics, ProviderTelemetryOperation, TelemetryResult};
pub use value::{
    checkout_ui_mode_name, error_kind, payment_event_type_name, payment_method_kind,
    payment_status_name, provider_name,
};
#[cfg(feature = "fraud")]
pub use value::{
    fraud_event_type_name, fraud_policy_mode_name, risk_decision_name, risk_level_name,
};
