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
        let cases = [
            (TelemetryOperation::PaymentCreate, "payrail.payment.create"),
            (TelemetryOperation::PaymentStatus, "payrail.payment.status"),
            (TelemetryOperation::PaymentRefund, "payrail.payment.refund"),
            (
                TelemetryOperation::PaymentCapture,
                "payrail.payment.capture",
            ),
            (TelemetryOperation::WebhookParse, "payrail.webhook.parse"),
            (
                TelemetryOperation::ProviderRequest,
                "payrail.provider.request",
            ),
            #[cfg(feature = "fraud")]
            (TelemetryOperation::FraudAssess, "payrail.fraud.assess"),
            #[cfg(feature = "fraud")]
            (
                TelemetryOperation::PaymentCreateWithRisk,
                "payrail.payment.create_with_risk",
            ),
            #[cfg(feature = "fraud")]
            (
                TelemetryOperation::FraudPolicyEvaluate,
                "payrail.fraud.policy.evaluate",
            ),
        ];

        for (operation, expected) in cases {
            assert_eq!(operation.as_str(), expected);
            assert_eq!(operation.as_ref(), expected);
        }
    }
}
