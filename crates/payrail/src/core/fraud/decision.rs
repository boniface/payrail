/// Normalized fraud risk level.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RiskLevel {
    /// Low fraud risk.
    #[default]
    Low,
    /// Medium fraud risk.
    Medium,
    /// High fraud risk.
    High,
    /// Critical fraud risk.
    Critical,
}

/// Normalized fraud decision ordered from least to most strict.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RiskDecision {
    /// Proceed normally.
    #[default]
    Allow,
    /// Require additional verification or authentication.
    Challenge,
    /// Hold for review.
    Review,
    /// Do not create the payment with the payment provider.
    Reject,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn risk_decision_order_is_strictness_order() {
        assert!(RiskDecision::Allow < RiskDecision::Challenge);
        assert!(RiskDecision::Challenge < RiskDecision::Review);
        assert!(RiskDecision::Review < RiskDecision::Reject);
    }

    #[test]
    fn defaults_are_low_risk_allow() {
        assert_eq!(RiskLevel::default(), RiskLevel::Low);
        assert_eq!(RiskDecision::default(), RiskDecision::Allow);
    }
}
