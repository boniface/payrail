/// Stable PayRail telemetry operation names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TelemetryOperation {
    /// Create a payment.
    PaymentCreate,
    /// Get payment status.
    PaymentStatus,
    /// Refund a payment.
    PaymentRefund,
    /// Capture a payment.
    PaymentCapture,
    /// Parse a webhook.
    WebhookParse,
    /// Provider HTTP request.
    ProviderRequest,
    /// Assess fraud risk.
    #[cfg(feature = "fraud")]
    FraudAssess,
    /// Create payment with risk assessment.
    #[cfg(feature = "fraud")]
    PaymentCreateWithRisk,
    /// Apply a fraud policy.
    #[cfg(feature = "fraud")]
    FraudPolicyEvaluate,
}

impl TelemetryOperation {
    /// Returns the stable span and operation name.
    #[inline]
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PaymentCreate => "payrail.payment.create",
            Self::PaymentStatus => "payrail.payment.status",
            Self::PaymentRefund => "payrail.payment.refund",
            Self::PaymentCapture => "payrail.payment.capture",
            Self::WebhookParse => "payrail.webhook.parse",
            Self::ProviderRequest => "payrail.provider.request",
            #[cfg(feature = "fraud")]
            Self::FraudAssess => "payrail.fraud.assess",
            #[cfg(feature = "fraud")]
            Self::PaymentCreateWithRisk => "payrail.payment.create_with_risk",
            #[cfg(feature = "fraud")]
            Self::FraudPolicyEvaluate => "payrail.fraud.policy.evaluate",
        }
    }
}

impl AsRef<str> for TelemetryOperation {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_names_match_contract() {
        assert_eq!(
            TelemetryOperation::PaymentCreate.as_str(),
            "payrail.payment.create"
        );
        assert_eq!(
            TelemetryOperation::ProviderRequest.as_str(),
            "payrail.provider.request"
        );
    }
}
