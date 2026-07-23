use crate::{
    CreatePaymentRequest, PaymentMethod, RiskAssessment, RiskContext, RiskDecision, RiskLevel,
    RiskReason, RiskReasonCode, RiskScore, VerificationStatus,
};
#[cfg(feature = "telemetry")]
use crate::{TelemetryOperation, emit_fraud_assessment, fraud_policy_mode_name};

/// How a fraud policy affects payment execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum FraudPolicyMode {
    /// Compute and return risk decisions without blocking provider execution.
    #[default]
    ObserveOnly,
    /// Enforce review and reject decisions before provider execution.
    Enforce,
}

/// Local fraud policy used before provider I/O.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FraudPolicy {
    mode: FraudPolicyMode,
    review_threshold: RiskScore,
    reject_threshold: RiskScore,
    require_verified_email_above: Option<i64>,
    require_verified_phone_above: Option<i64>,
    allow_review_authorization: bool,
}

impl FraudPolicy {
    /// Creates a policy in observe-only mode with conservative default thresholds.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets observe-only mode.
    #[inline]
    #[must_use]
    pub const fn observe_only(mut self) -> Self {
        self.mode = FraudPolicyMode::ObserveOnly;
        self
    }

    /// Sets enforce mode.
    #[inline]
    #[must_use]
    pub const fn enforce(mut self) -> Self {
        self.mode = FraudPolicyMode::Enforce;
        self
    }

    /// Sets the score at which payments require review.
    #[inline]
    #[must_use]
    pub const fn review_threshold(mut self, threshold: RiskScore) -> Self {
        self.review_threshold = threshold;
        self
    }

    /// Sets the score at which payments are rejected.
    #[inline]
    #[must_use]
    pub const fn reject_threshold(mut self, threshold: RiskScore) -> Self {
        self.reject_threshold = threshold;
        self
    }

    /// Requires verified email for payments above a minor-unit amount.
    #[inline]
    #[must_use]
    pub const fn require_verified_email_above(mut self, amount: i64) -> Self {
        self.require_verified_email_above = Some(amount);
        self
    }

    /// Requires verified phone for payments above a minor-unit amount.
    #[inline]
    #[must_use]
    pub const fn require_verified_phone_above(mut self, amount: i64) -> Self {
        self.require_verified_phone_above = Some(amount);
        self
    }

    /// Allows provider authorization while a payment is queued for review.
    #[inline]
    #[must_use]
    pub const fn allow_review_authorization(mut self, allow: bool) -> Self {
        self.allow_review_authorization = allow;
        self
    }

    /// Returns the execution mode.
    #[inline]
    #[must_use]
    pub const fn mode(&self) -> FraudPolicyMode {
        self.mode
    }

    /// Returns the review threshold.
    #[inline]
    #[must_use]
    pub const fn review_threshold_value(&self) -> RiskScore {
        self.review_threshold
    }

    /// Returns the reject threshold.
    #[inline]
    #[must_use]
    pub const fn reject_threshold_value(&self) -> RiskScore {
        self.reject_threshold
    }

    /// Returns whether review decisions may continue to provider authorization.
    #[inline]
    #[must_use]
    pub const fn allows_review_authorization(&self) -> bool {
        self.allow_review_authorization
    }

    /// Assesses payment risk using local policy rules.
    #[must_use]
    pub fn assess_request(&self, request: &CreatePaymentRequest) -> RiskAssessment {
        #[cfg(feature = "telemetry")]
        {
            let span = tracing::info_span!(
                "payrail.fraud.policy.evaluate",
                "payrail.operation" = TelemetryOperation::FraudPolicyEvaluate.as_str(),
                "payrail.policy_mode" = fraud_policy_mode_name(self.mode())
            );
            let _guard = span.enter();
            let assessment = self.assess_request_inner(request);
            emit_fraud_assessment(TelemetryOperation::FraudPolicyEvaluate, &assessment, "none");
            assessment
        }

        #[cfg(not(feature = "telemetry"))]
        self.assess_request_inner(request)
    }

    fn assess_request_inner(&self, request: &CreatePaymentRequest) -> RiskAssessment {
        let mut score = 0;
        let mut reasons = Vec::new();

        if let Some(context) = request.risk_context() {
            Self::score_context(context, &mut score, &mut reasons);
        }

        if matches!(
            request.payment_method(),
            PaymentMethod::Crypto(_) | PaymentMethod::Stablecoin(_)
        ) {
            score = score.max(300);
        }

        let email_challenge =
            self.requires_verification(request, self.require_verified_email_above, |context| {
                context.email_status()
            });
        if email_challenge {
            reasons.push(RiskReason::new(RiskReasonCode::EmailUnverified));
            score = score.max(450);
        }

        let phone_challenge =
            self.requires_verification(request, self.require_verified_phone_above, |context| {
                context.phone_status()
            });
        if phone_challenge {
            reasons.push(RiskReason::new(RiskReasonCode::PhoneUnverified));
            score = score.max(450);
        }

        let mut decision = if score >= self.reject_threshold.value() {
            RiskDecision::Reject
        } else if score >= self.review_threshold.value() {
            RiskDecision::Review
        } else {
            RiskDecision::Allow
        };
        if email_challenge || phone_challenge {
            decision = decision.max(RiskDecision::Challenge);
        }

        let level = Self::risk_level(score);
        let normalized_score = RiskScore::new(score)
            .expect("local policy score is bounded to the RiskScore valid range");

        reasons.into_iter().fold(
            RiskAssessment::new(decision)
                .with_score(normalized_score)
                .with_level(level),
            |assessment, reason| assessment.with_reason(reason),
        )
    }

    /// Returns whether provider execution may proceed for an assessment.
    #[inline]
    #[must_use]
    pub const fn allows_payment_creation(&self, decision: RiskDecision) -> bool {
        match self.mode {
            FraudPolicyMode::ObserveOnly => true,
            FraudPolicyMode::Enforce => {
                matches!(decision, RiskDecision::Allow | RiskDecision::Challenge)
                    || matches!(decision, RiskDecision::Review) && self.allow_review_authorization
            }
        }
    }

    fn score_context(context: &RiskContext, score: &mut u16, reasons: &mut Vec<RiskReason>) {
        if matches!(
            context.customer_segment(),
            Some(crate::CustomerSegment::HighRisk)
        ) {
            Self::apply_score(score, reasons, 700, RiskReasonCode::ProviderRule);
        }

        if matches!(context.account_age_days(), Some(days) if days < 7) {
            Self::apply_score(score, reasons, 450, RiskReasonCode::AccountAge);
        }

        if let Some(device) = context.device()
            && (device.seen_before() == Some(false) || device.recently_changed() == Some(true))
        {
            Self::apply_score(score, reasons, 550, RiskReasonCode::DeviceMismatch);
        }

        if let Some(network) = context.network() {
            if network.proxy_detected() == Some(true)
                || network.vpn_detected() == Some(true)
                || network.tor_detected() == Some(true)
            {
                Self::apply_score(score, reasons, 650, RiskReasonCode::ProxyOrVpn);
            }

            if Self::country_mismatch(context) {
                Self::apply_score(score, reasons, 550, RiskReasonCode::GeoMismatch);
            }
        }

        if let Some(velocity) = context.velocity() {
            if matches!(velocity.attempts_last_10_minutes(), Some(attempts) if attempts >= 5)
                || matches!(velocity.attempts_last_hour(), Some(attempts) if attempts >= 10)
                || matches!(velocity.failed_payments_last_day(), Some(failures) if failures >= 5)
            {
                Self::apply_score(score, reasons, 750, RiskReasonCode::VelocityExceeded);
            }

            if matches!(velocity.chargebacks_last_90_days(), Some(chargebacks) if chargebacks > 0) {
                Self::apply_score(score, reasons, 850, RiskReasonCode::ProviderRule);
            }
        }
    }

    fn apply_score(
        score: &mut u16,
        reasons: &mut Vec<RiskReason>,
        candidate: u16,
        code: RiskReasonCode,
    ) {
        *score = (*score).max(candidate);
        if !reasons.iter().any(|reason| reason.code() == &code) {
            reasons.push(RiskReason::new(code));
        }
    }

    fn country_mismatch(context: &RiskContext) -> bool {
        let Some(network) = context.network() else {
            return false;
        };

        let billing_shipping_mismatch = network
            .billing_country()
            .zip(network.shipping_country())
            .is_some_and(|(billing, shipping)| billing != shipping);
        let ip_billing_mismatch = network
            .ip_country()
            .zip(network.billing_country())
            .is_some_and(|(ip, billing)| ip != billing);

        billing_shipping_mismatch || ip_billing_mismatch
    }

    fn requires_verification(
        &self,
        request: &CreatePaymentRequest,
        threshold: Option<i64>,
        status: impl FnOnce(&RiskContext) -> VerificationStatus,
    ) -> bool {
        let Some(context) = request.risk_context() else {
            return false;
        };

        if matches!(status(context), VerificationStatus::Verified) {
            return false;
        }

        threshold.is_some_and(|amount| request.amount().amount().value() > amount)
    }

    const fn risk_level(score: u16) -> RiskLevel {
        match score {
            0..=249 => RiskLevel::Low,
            250..=499 => RiskLevel::Medium,
            500..=749 => RiskLevel::High,
            750..=1000 => RiskLevel::Critical,
            1001..=u16::MAX => RiskLevel::Critical,
        }
    }
}

impl Default for FraudPolicy {
    fn default() -> Self {
        Self {
            mode: FraudPolicyMode::ObserveOnly,
            review_threshold: RiskScore::new(600)
                .expect("default review threshold should be valid"),
            reject_threshold: RiskScore::new(850)
                .expect("default reject threshold should be valid"),
            require_verified_email_above: None,
            require_verified_phone_above: None,
            allow_review_authorization: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        CountryCode, Money, NetworkRiskContext, PaymentMethod, RiskContext, RiskDecision,
        VelocityRiskContext, VerificationStatus,
    };

    use super::*;

    fn request_with_context(context: RiskContext) -> CreatePaymentRequest {
        CreatePaymentRequest::builder()
            .amount(Money::new_minor(10_000, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::card())
            .risk_context(context)
            .build()
            .expect("request should be valid")
    }

    #[test]
    fn default_policy_is_observe_only() {
        let policy = FraudPolicy::default();

        assert_eq!(policy.mode(), FraudPolicyMode::ObserveOnly);
        assert!(policy.allows_payment_creation(RiskDecision::Reject));
    }

    #[test]
    fn velocity_can_reject_payment() {
        let context = RiskContext::new().with_velocity(
            VelocityRiskContext::new()
                .with_attempts_last_10_minutes(5)
                .with_chargebacks_last_90_days(1),
        );

        let assessment = FraudPolicy::new()
            .enforce()
            .assess_request(&request_with_context(context));

        assert_eq!(assessment.decision(), RiskDecision::Reject);
        assert_eq!(assessment.level(), Some(RiskLevel::Critical));
        assert!(
            assessment
                .reasons()
                .iter()
                .any(|reason| reason.code() == &RiskReasonCode::VelocityExceeded)
        );
    }

    #[test]
    fn network_mismatch_can_trigger_review() {
        let context = RiskContext::new().with_network(
            NetworkRiskContext::new()
                .with_ip_country(CountryCode::new("US").expect("country should be valid"))
                .with_billing_country(CountryCode::new("US").expect("country should be valid"))
                .with_shipping_country(CountryCode::new("CA").expect("country should be valid")),
        );

        let assessment = FraudPolicy::new()
            .review_threshold(RiskScore::new(500).expect("score should be valid"))
            .assess_request(&request_with_context(context));

        assert_eq!(assessment.decision(), RiskDecision::Review);
        assert_eq!(assessment.level(), Some(RiskLevel::High));
    }

    #[test]
    fn verification_requirement_challenges_high_value_payment() {
        let context = RiskContext::new().with_email_status(VerificationStatus::Pending);
        let assessment = FraudPolicy::new()
            .require_verified_email_above(5_000)
            .assess_request(&request_with_context(context));

        assert_eq!(assessment.decision(), RiskDecision::Challenge);
        assert!(
            assessment
                .reasons()
                .iter()
                .any(|reason| reason.code() == &RiskReasonCode::EmailUnverified)
        );
    }
}
