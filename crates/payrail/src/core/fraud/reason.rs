/// Normalized fraud risk reason code.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RiskReasonCode {
    /// Velocity threshold was exceeded.
    VelocityExceeded,
    /// Device signal did not match expected behavior.
    DeviceMismatch,
    /// Geography did not match expected behavior.
    GeoMismatch,
    /// Proxy or VPN signal was present.
    ProxyOrVpn,
    /// Account age was below policy threshold.
    AccountAge,
    /// Email was not verified.
    EmailUnverified,
    /// Phone was not verified.
    PhoneUnverified,
    /// Country was considered high risk by policy.
    HighRiskCountry,
    /// Payment method was considered high risk by policy.
    HighRiskPaymentMethod,
    /// Provider rule produced the reason.
    ProviderRule,
    /// Provider model produced the reason.
    ProviderModel,
    /// Manual review produced the reason.
    ManualReview,
    /// Reason not modeled directly.
    Other(String),
}

/// Normalized fraud risk reason.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RiskReason {
    code: RiskReasonCode,
    message: Option<String>,
}

impl RiskReason {
    /// Creates a risk reason with no message.
    #[inline]
    #[must_use]
    pub const fn new(code: RiskReasonCode) -> Self {
        Self {
            code,
            message: None,
        }
    }

    /// Adds a safe, redacted message.
    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Returns the reason code.
    #[inline]
    #[must_use]
    pub const fn code(&self) -> &RiskReasonCode {
        &self.code
    }

    /// Returns the safe, redacted message.
    #[inline]
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn risk_reason_accessors_return_fields() {
        let reason =
            RiskReason::new(RiskReasonCode::VelocityExceeded).with_message("too many attempts");

        assert_eq!(reason.code(), &RiskReasonCode::VelocityExceeded);
        assert_eq!(reason.message(), Some("too many attempts"));
    }
}
