use crate::{
    FraudProvider, FraudProviderReference, ReviewRequest, RiskDecision, RiskLevel, RiskReason,
    RiskScore,
};

/// Provider-neutral fraud risk assessment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskAssessment {
    provider: Option<FraudProvider>,
    provider_reference: Option<FraudProviderReference>,
    decision: RiskDecision,
    score: Option<RiskScore>,
    level: Option<RiskLevel>,
    reasons: Vec<RiskReason>,
    review: Option<ReviewRequest>,
}

impl RiskAssessment {
    /// Creates a fraud risk assessment for a decision.
    #[inline]
    #[must_use]
    pub fn new(decision: RiskDecision) -> Self {
        Self {
            provider: None,
            provider_reference: None,
            decision,
            score: None,
            level: None,
            reasons: Vec::new(),
            review: None,
        }
    }

    /// Sets the fraud provider.
    #[must_use]
    pub fn with_provider(mut self, provider: FraudProvider) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Sets the fraud provider reference.
    #[must_use]
    pub fn with_provider_reference(mut self, reference: FraudProviderReference) -> Self {
        self.provider_reference = Some(reference);
        self
    }

    /// Sets the normalized risk score.
    #[must_use]
    pub const fn with_score(mut self, score: RiskScore) -> Self {
        self.score = Some(score);
        self
    }

    /// Sets the normalized risk level.
    #[must_use]
    pub const fn with_level(mut self, level: RiskLevel) -> Self {
        self.level = Some(level);
        self
    }

    /// Adds a fraud risk reason.
    #[must_use]
    pub fn with_reason(mut self, reason: RiskReason) -> Self {
        self.reasons.push(reason);
        self
    }

    /// Sets the review request.
    #[must_use]
    pub fn with_review(mut self, review: ReviewRequest) -> Self {
        self.review = Some(review);
        self
    }

    /// Returns the fraud provider.
    #[inline]
    #[must_use]
    pub const fn provider(&self) -> Option<&FraudProvider> {
        self.provider.as_ref()
    }

    /// Returns the fraud provider reference.
    #[inline]
    #[must_use]
    pub const fn provider_reference(&self) -> Option<&FraudProviderReference> {
        self.provider_reference.as_ref()
    }

    /// Returns the fraud decision.
    #[inline]
    #[must_use]
    pub const fn decision(&self) -> RiskDecision {
        self.decision
    }

    /// Returns the normalized risk score.
    #[inline]
    #[must_use]
    pub const fn score(&self) -> Option<RiskScore> {
        self.score
    }

    /// Returns the normalized risk level.
    #[inline]
    #[must_use]
    pub const fn level(&self) -> Option<RiskLevel> {
        self.level
    }

    /// Returns the safe risk reasons.
    #[inline]
    #[must_use]
    pub fn reasons(&self) -> &[RiskReason] {
        &self.reasons
    }

    /// Returns the review request.
    #[inline]
    #[must_use]
    pub const fn review(&self) -> Option<&ReviewRequest> {
        self.review.as_ref()
    }
}

impl Default for RiskAssessment {
    fn default() -> Self {
        Self::new(RiskDecision::Allow)
    }
}

#[cfg(test)]
mod tests {
    use crate::{RiskReasonCode, VerificationStatus};

    use super::*;

    #[test]
    fn risk_assessment_accessors_return_configured_values() {
        let provider_reference =
            FraudProviderReference::new("risk_123").expect("reference should be valid");
        let review_reference =
            FraudProviderReference::new("review_123").expect("reference should be valid");
        let score = RiskScore::new(750).expect("score should be valid");
        let reason = RiskReason::new(RiskReasonCode::VelocityExceeded);
        let review = ReviewRequest::new()
            .with_provider_reference(review_reference)
            .with_message("manual review required");

        let assessment = RiskAssessment::new(RiskDecision::Review)
            .with_provider(FraudProvider::StripeRadar)
            .with_provider_reference(provider_reference)
            .with_score(score)
            .with_level(RiskLevel::High)
            .with_reason(reason)
            .with_review(review);

        assert_eq!(assessment.provider(), Some(&FraudProvider::StripeRadar));
        assert_eq!(
            assessment
                .provider_reference()
                .expect("reference should exist")
                .as_str(),
            "risk_123"
        );
        assert_eq!(assessment.decision(), RiskDecision::Review);
        assert_eq!(assessment.score(), Some(score));
        assert_eq!(assessment.level(), Some(RiskLevel::High));
        assert_eq!(assessment.reasons().len(), 1);
        assert_eq!(
            assessment.review().expect("review should exist").message(),
            Some("manual review required")
        );
    }

    #[test]
    fn risk_assessment_default_allows_payment() {
        let assessment = RiskAssessment::default();

        assert_eq!(assessment.decision(), RiskDecision::Allow);
        assert!(assessment.provider().is_none());
        assert!(assessment.reasons().is_empty());
        assert_eq!(
            VerificationStatus::default(),
            VerificationStatus::NotProvided
        );
    }
}
