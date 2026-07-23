use opentelemetry::{
    KeyValue,
    metrics::{Counter, Histogram, Meter},
};

use crate::{
    PaymentEventType, PaymentProvider, TELEMETRY_METRIC_PAYMENT_REQUESTS_TOTAL,
    TELEMETRY_METRIC_PROVIDER_REQUEST_DURATION_MS, TELEMETRY_METRIC_PROVIDER_REQUESTS_TOTAL,
    TELEMETRY_METRIC_WEBHOOKS_TOTAL, TelemetryOperation, payment_event_type_name, provider_name,
};
#[cfg(feature = "fraud")]
use crate::{
    RiskAssessment, RiskDecision, TELEMETRY_METRIC_FRAUD_ASSESSMENTS_TOTAL,
    TELEMETRY_METRIC_FRAUD_POLICY_BLOCKS_TOTAL, risk_decision_name, risk_level_name,
};

const PROVIDER_UNRESOLVED: &str = "unresolved";
const EVENT_UNKNOWN: &str = "unknown";

/// Low-cardinality operation result for OpenTelemetry metric labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TelemetryResult {
    /// The operation completed successfully.
    #[default]
    Ok,
    /// The operation completed with an error.
    Error,
}

impl TelemetryResult {
    /// Returns the stable metric-label value.
    #[inline]
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
        }
    }
}

impl AsRef<str> for TelemetryResult {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Low-cardinality provider operation label for OpenTelemetry metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProviderTelemetryOperation {
    /// Create a provider payment.
    #[default]
    CreatePayment,
    /// Retrieve provider payment status.
    GetPaymentStatus,
    /// Refund a provider payment.
    RefundPayment,
    /// Capture an authorized provider payment.
    CapturePayment,
    /// Parse a provider webhook.
    ParseWebhook,
    /// Verify a provider webhook signature.
    VerifyWebhookSignature,
    /// Fetch an OAuth or provider access token.
    FetchAccessToken,
    /// Retrieve a Stripe checkout session.
    CheckoutSessionRetrieve,
}

impl ProviderTelemetryOperation {
    /// Returns the stable metric-label value.
    #[inline]
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CreatePayment => "create_payment",
            Self::GetPaymentStatus => "get_payment_status",
            Self::RefundPayment => "refund_payment",
            Self::CapturePayment => "capture_payment",
            Self::ParseWebhook => "parse_webhook",
            Self::VerifyWebhookSignature => "verify_webhook_signature",
            Self::FetchAccessToken => "fetch_access_token",
            Self::CheckoutSessionRetrieve => "checkout_session_retrieve",
        }
    }
}

impl AsRef<str> for ProviderTelemetryOperation {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Optional OpenTelemetry metric instruments for PayRail applications.
///
/// This type uses only the OpenTelemetry API crate. Applications still own SDK setup, exporters,
/// readers, resource attributes, and collector configuration.
#[derive(Debug, Clone)]
pub struct PayRailOtelMetrics {
    payment_requests_total: Counter<u64>,
    provider_requests_total: Counter<u64>,
    provider_request_duration_ms: Histogram<f64>,
    webhooks_total: Counter<u64>,
    #[cfg(feature = "fraud")]
    fraud_assessments_total: Counter<u64>,
    #[cfg(feature = "fraud")]
    fraud_policy_blocks_total: Counter<u64>,
}

impl PayRailOtelMetrics {
    /// Creates PayRail metric instruments from an application-owned meter.
    #[must_use]
    pub fn new(meter: &Meter) -> Self {
        Self {
            payment_requests_total: meter
                .u64_counter(TELEMETRY_METRIC_PAYMENT_REQUESTS_TOTAL)
                .with_description("PayRail payment requests")
                .build(),
            provider_requests_total: meter
                .u64_counter(TELEMETRY_METRIC_PROVIDER_REQUESTS_TOTAL)
                .with_description("PayRail provider requests")
                .build(),
            provider_request_duration_ms: meter
                .f64_histogram(TELEMETRY_METRIC_PROVIDER_REQUEST_DURATION_MS)
                .with_description("PayRail provider request duration")
                .with_unit("ms")
                .build(),
            webhooks_total: meter
                .u64_counter(TELEMETRY_METRIC_WEBHOOKS_TOTAL)
                .with_description("PayRail parsed webhooks")
                .build(),
            #[cfg(feature = "fraud")]
            fraud_assessments_total: meter
                .u64_counter(TELEMETRY_METRIC_FRAUD_ASSESSMENTS_TOTAL)
                .with_description("PayRail fraud assessments")
                .build(),
            #[cfg(feature = "fraud")]
            fraud_policy_blocks_total: meter
                .u64_counter(TELEMETRY_METRIC_FRAUD_POLICY_BLOCKS_TOTAL)
                .with_description("PayRail fraud policy blocks")
                .build(),
        }
    }

    /// Records a payment request.
    pub fn record_payment_request(
        &self,
        provider: Option<&PaymentProvider>,
        operation: TelemetryOperation,
        result: TelemetryResult,
    ) {
        let attributes = operation_attributes(provider, operation, result);
        self.payment_requests_total.add(1, &attributes);
    }

    /// Records a provider request.
    pub fn record_provider_request(
        &self,
        provider: &PaymentProvider,
        operation: ProviderTelemetryOperation,
        result: TelemetryResult,
    ) {
        let attributes = provider_operation_attributes(provider, operation, result);
        self.provider_requests_total.add(1, &attributes);
    }

    /// Records a provider request duration in milliseconds.
    pub fn record_provider_request_duration_ms(
        &self,
        provider: &PaymentProvider,
        operation: ProviderTelemetryOperation,
        result: TelemetryResult,
        duration_ms: f64,
    ) {
        if !duration_ms.is_finite() || duration_ms.is_sign_negative() {
            return;
        }

        let attributes = provider_operation_attributes(provider, operation, result);
        self.provider_request_duration_ms
            .record(duration_ms, &attributes);
    }

    /// Records a webhook parse result.
    pub fn record_webhook(
        &self,
        provider: &PaymentProvider,
        event_type: Option<PaymentEventType>,
        result: TelemetryResult,
    ) {
        let attributes = webhook_attributes(provider, event_type, result);
        self.webhooks_total.add(1, &attributes);
    }

    /// Records a fraud assessment result.
    #[cfg(feature = "fraud")]
    pub fn record_fraud_assessment(&self, assessment: &RiskAssessment, result: TelemetryResult) {
        let attributes = fraud_assessment_attributes(assessment, result);
        self.fraud_assessments_total.add(1, &attributes);
    }

    /// Records a fraud policy block.
    #[cfg(feature = "fraud")]
    pub fn record_fraud_policy_block(&self, decision: RiskDecision) {
        let attributes = [KeyValue::new("risk_decision", risk_decision_name(decision))];
        self.fraud_policy_blocks_total.add(1, &attributes);
    }
}

fn operation_attributes(
    provider: Option<&PaymentProvider>,
    operation: TelemetryOperation,
    result: TelemetryResult,
) -> [KeyValue; 3] {
    [
        KeyValue::new(
            "provider",
            provider.map_or(PROVIDER_UNRESOLVED, provider_name),
        ),
        KeyValue::new("operation", operation.as_str()),
        KeyValue::new("result", result.as_str()),
    ]
}

fn provider_operation_attributes(
    provider: &PaymentProvider,
    operation: ProviderTelemetryOperation,
    result: TelemetryResult,
) -> [KeyValue; 3] {
    [
        KeyValue::new("provider", provider_name(provider)),
        KeyValue::new("operation", operation.as_str()),
        KeyValue::new("result", result.as_str()),
    ]
}

fn webhook_attributes(
    provider: &PaymentProvider,
    event_type: Option<PaymentEventType>,
    result: TelemetryResult,
) -> [KeyValue; 3] {
    [
        KeyValue::new("provider", provider_name(provider)),
        KeyValue::new(
            "event_type",
            event_type.map_or(EVENT_UNKNOWN, payment_event_type_name),
        ),
        KeyValue::new("result", result.as_str()),
    ]
}

#[cfg(feature = "fraud")]
fn fraud_assessment_attributes(
    assessment: &RiskAssessment,
    result: TelemetryResult,
) -> [KeyValue; 3] {
    [
        KeyValue::new("risk_decision", risk_decision_name(assessment.decision())),
        KeyValue::new(
            "risk_level",
            assessment.level().map_or(EVENT_UNKNOWN, risk_level_name),
        ),
        KeyValue::new("result", result.as_str()),
    ]
}

#[cfg(test)]
mod tests {
    use opentelemetry::global;

    #[cfg(feature = "fraud")]
    use crate::RiskLevel;

    use super::*;

    #[test]
    fn telemetry_result_values_are_stable() {
        assert_eq!(TelemetryResult::Ok.as_str(), "ok");
        assert_eq!(TelemetryResult::Error.as_ref(), "error");
    }

    #[test]
    fn provider_operation_values_are_stable() {
        let cases = [
            (ProviderTelemetryOperation::CreatePayment, "create_payment"),
            (
                ProviderTelemetryOperation::GetPaymentStatus,
                "get_payment_status",
            ),
            (ProviderTelemetryOperation::RefundPayment, "refund_payment"),
            (
                ProviderTelemetryOperation::CapturePayment,
                "capture_payment",
            ),
            (ProviderTelemetryOperation::ParseWebhook, "parse_webhook"),
            (
                ProviderTelemetryOperation::VerifyWebhookSignature,
                "verify_webhook_signature",
            ),
            (
                ProviderTelemetryOperation::FetchAccessToken,
                "fetch_access_token",
            ),
            (
                ProviderTelemetryOperation::CheckoutSessionRetrieve,
                "checkout_session_retrieve",
            ),
        ];

        for (operation, expected) in cases {
            assert_eq!(operation.as_str(), expected);
            assert_eq!(operation.as_ref(), expected);
        }
    }

    #[test]
    fn payment_attributes_do_not_include_references() {
        let attributes = operation_attributes(
            Some(&PaymentProvider::Other("raw-provider".to_owned())),
            TelemetryOperation::PaymentCreate,
            TelemetryResult::Ok,
        );

        assert!(attributes.iter().any(|attribute| {
            attribute.key.as_str() == "provider" && attribute.value.as_str() == "other"
        }));
        assert!(!format!("{attributes:?}").contains("raw-provider"));
    }

    #[test]
    fn metrics_can_be_created_from_global_meter() {
        let meter = global::meter("payrail.test");
        let metrics = PayRailOtelMetrics::new(&meter);

        metrics.record_payment_request(
            Some(&PaymentProvider::Stripe),
            TelemetryOperation::PaymentCreate,
            TelemetryResult::Ok,
        );
        metrics.record_provider_request(
            &PaymentProvider::Stripe,
            ProviderTelemetryOperation::CreatePayment,
            TelemetryResult::Ok,
        );
        metrics.record_provider_request_duration_ms(
            &PaymentProvider::Stripe,
            ProviderTelemetryOperation::CreatePayment,
            TelemetryResult::Ok,
            12.5,
        );
        metrics.record_webhook(
            &PaymentProvider::Stripe,
            Some(PaymentEventType::PaymentSucceeded),
            TelemetryResult::Ok,
        );
        metrics.record_payment_request(
            None,
            TelemetryOperation::PaymentRefund,
            TelemetryResult::Error,
        );
        metrics.record_webhook(&PaymentProvider::Stripe, None, TelemetryResult::Error);
    }

    #[cfg(feature = "fraud")]
    #[test]
    fn fraud_attributes_are_low_cardinality() {
        let reference = crate::FraudProviderReference::new("risk_secret_reference")
            .expect("reference should be valid");
        let assessment = RiskAssessment::new(RiskDecision::Reject)
            .with_provider_reference(reference)
            .with_level(RiskLevel::Critical);
        let attributes = fraud_assessment_attributes(&assessment, TelemetryResult::Ok);

        assert!(attributes.iter().any(|attribute| {
            attribute.key.as_str() == "risk_decision" && attribute.value.as_str() == "reject"
        }));
        assert!(!format!("{attributes:?}").contains("risk_secret_reference"));
    }

    #[cfg(feature = "fraud")]
    #[test]
    fn fraud_metrics_can_be_recorded() {
        let meter = global::meter("payrail.fraud.metrics.test");
        let metrics = PayRailOtelMetrics::new(&meter);
        let assessment = RiskAssessment::new(RiskDecision::Review).with_level(RiskLevel::High);

        metrics.record_fraud_assessment(&assessment, TelemetryResult::Error);
        metrics.record_fraud_policy_block(RiskDecision::Reject);
    }

    #[test]
    fn invalid_duration_is_ignored() {
        let meter = global::meter("payrail.duration.test");
        let metrics = PayRailOtelMetrics::new(&meter);

        metrics.record_provider_request_duration_ms(
            &PaymentProvider::Stripe,
            ProviderTelemetryOperation::CreatePayment,
            TelemetryResult::Ok,
            f64::NAN,
        );
    }
}
