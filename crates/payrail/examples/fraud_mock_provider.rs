use payrail::{
    CreatePaymentRequest, FraudProvider, FraudProviderReference, Money, PaymentError,
    PaymentMethod, RiskAssessment, RiskDecision, RiskLevel, RiskScore,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MockFraudOutcome {
    Allow,
    Challenge,
    Review,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MockFraudProvider {
    outcome: MockFraudOutcome,
}

impl MockFraudProvider {
    const fn new(outcome: MockFraudOutcome) -> Self {
        Self { outcome }
    }

    fn assess(&self, request: &CreatePaymentRequest) -> Result<RiskAssessment, PaymentError> {
        let _ = request.reference();
        match self.outcome {
            MockFraudOutcome::Allow => mock_assessment(RiskDecision::Allow, 50, RiskLevel::Low),
            MockFraudOutcome::Challenge => {
                mock_assessment(RiskDecision::Challenge, 450, RiskLevel::Medium)
            }
            MockFraudOutcome::Review => mock_assessment(RiskDecision::Review, 650, RiskLevel::High),
            MockFraudOutcome::Reject => {
                mock_assessment(RiskDecision::Reject, 900, RiskLevel::Critical)
            }
        }
    }
}

fn mock_assessment(
    decision: RiskDecision,
    score: u16,
    level: RiskLevel,
) -> Result<RiskAssessment, PaymentError> {
    Ok(RiskAssessment::new(decision)
        .with_provider(FraudProvider::Other("mock-fraud".to_owned()))
        .with_provider_reference(FraudProviderReference::new("mock_assessment_123")?)
        .with_score(RiskScore::new(score)?)
        .with_level(level))
}

fn main() -> Result<(), PaymentError> {
    let request = CreatePaymentRequest::builder()
        .amount(Money::new_minor(1_000, "USD")?)
        .reference("ORDER-1")?
        .payment_method(PaymentMethod::card())
        .build()?;

    let outcomes = [
        (MockFraudOutcome::Allow, RiskDecision::Allow),
        (MockFraudOutcome::Challenge, RiskDecision::Challenge),
        (MockFraudOutcome::Review, RiskDecision::Review),
        (MockFraudOutcome::Reject, RiskDecision::Reject),
    ];

    for (outcome, decision) in outcomes {
        let assessment = MockFraudProvider::new(outcome).assess(&request)?;
        assert_eq!(assessment.decision(), decision);
    }

    Ok(())
}
