use crate::{PaymentSession, RiskAssessment};

/// Payment creation result with the fraud decision that governed execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskAwarePaymentSession {
    payment: Option<PaymentSession>,
    assessment: RiskAssessment,
}

impl RiskAwarePaymentSession {
    /// Creates a risk-aware payment result.
    #[inline]
    #[must_use]
    pub const fn new(payment: Option<PaymentSession>, assessment: RiskAssessment) -> Self {
        Self {
            payment,
            assessment,
        }
    }

    /// Returns the created payment when policy allowed provider execution.
    #[inline]
    #[must_use]
    pub const fn payment(&self) -> Option<&PaymentSession> {
        self.payment.as_ref()
    }

    /// Returns the fraud assessment.
    #[inline]
    #[must_use]
    pub const fn assessment(&self) -> &RiskAssessment {
        &self.assessment
    }

    /// Consumes the result and returns its parts.
    #[inline]
    #[must_use]
    pub fn into_parts(self) -> (Option<PaymentSession>, RiskAssessment) {
        (self.payment, self.assessment)
    }
}

#[cfg(test)]
mod tests {
    use crate::{RiskAssessment, RiskDecision};

    use super::*;

    #[test]
    fn risk_aware_session_can_represent_policy_rejection() {
        let assessment = RiskAssessment::new(RiskDecision::Reject);
        let session = RiskAwarePaymentSession::new(None, assessment);

        assert!(session.payment().is_none());
        assert_eq!(session.assessment().decision(), RiskDecision::Reject);
    }
}
