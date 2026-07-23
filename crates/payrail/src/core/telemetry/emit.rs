use crate::{PaymentError, TelemetryOperation, error_kind};
#[cfg(any(feature = "lipila", feature = "paypal", feature = "stripe"))]
use crate::{PaymentProvider, provider_name};
#[cfg(feature = "fraud")]
use crate::{RiskAssessment, risk_decision_name, risk_level_name};

/// Emits a low-cardinality operation result event.
pub(crate) fn emit_result<T>(
    operation: TelemetryOperation,
    result: &Result<T, PaymentError>,
    message: &'static str,
) {
    match result {
        Ok(_) => tracing::debug!(
            "payrail.operation" = operation.as_str(),
            "payrail.result" = "ok",
            "{message}"
        ),
        Err(error) => tracing::warn!(
            "payrail.operation" = operation.as_str(),
            "payrail.result" = "error",
            "payrail.error_kind" = error_kind(error),
            "{message}"
        ),
    }
}

/// Emits a low-cardinality provider request result event.
#[cfg(any(feature = "lipila", feature = "paypal", feature = "stripe"))]
pub(crate) fn emit_provider_request_result(
    provider: &PaymentProvider,
    operation: &'static str,
    http_status: u16,
    success: bool,
) {
    tracing::debug!(
        "payrail.provider" = provider_name(provider),
        "payrail.operation" = operation,
        "payrail.http_status" = http_status,
        "payrail.result" = if success { "ok" } else { "error" },
        "provider request completed"
    );
}

/// Emits a low-cardinality fraud risk assessment event.
#[cfg(feature = "fraud")]
pub(crate) fn emit_fraud_assessment(
    operation: TelemetryOperation,
    assessment: &RiskAssessment,
    provider_io: &'static str,
) {
    tracing::debug!(
        "payrail.operation" = operation.as_str(),
        "payrail.risk_decision" = risk_decision_name(assessment.decision()),
        "payrail.risk_level" = assessment.level().map_or("unknown", risk_level_name),
        "payrail.reason_count" = assessment.reasons().len(),
        "payrail.provider_io" = provider_io,
        "fraud assessment completed"
    );
}

#[cfg(test)]
mod tests {
    use crate::{PaymentError, TelemetryOperation};
    #[cfg(feature = "fraud")]
    use crate::{RiskAssessment, RiskDecision};

    use super::*;

    #[test]
    fn emits_success_and_error_results() {
        let ok: Result<(), PaymentError> = Ok(());
        let error: Result<(), PaymentError> = Err(PaymentError::AuthenticationFailed);

        emit_result(TelemetryOperation::PaymentCreate, &ok, "ok result");
        emit_result(TelemetryOperation::PaymentCreate, &error, "error result");
    }

    #[cfg(any(feature = "lipila", feature = "paypal", feature = "stripe"))]
    #[test]
    fn emits_provider_request_results() {
        emit_provider_request_result(&crate::PaymentProvider::Stripe, "create_payment", 200, true);
        emit_provider_request_result(
            &crate::PaymentProvider::Stripe,
            "create_payment",
            500,
            false,
        );
    }

    #[cfg(feature = "fraud")]
    #[test]
    fn emits_fraud_assessment_result() {
        let assessment = RiskAssessment::new(RiskDecision::Reject);

        emit_fraud_assessment(TelemetryOperation::FraudAssess, &assessment, "skipped");
    }
}
