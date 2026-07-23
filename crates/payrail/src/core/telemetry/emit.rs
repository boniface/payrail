use crate::{PaymentError, TelemetryOperation, error_kind};
#[cfg(any(feature = "lipila", feature = "paypal", feature = "stripe"))]
use crate::{PaymentProvider, provider_name};

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

#[cfg(test)]
mod tests {
    use crate::{PaymentError, TelemetryOperation};

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
}
