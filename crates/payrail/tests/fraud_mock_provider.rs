#![cfg(feature = "fraud")]

use payrail::{
    CreatePaymentRequest, FraudProvider, FraudProviderReference, Money, PaymentError,
    PaymentMethod, PaymentProvider, ProviderErrorDetails, RiskAssessment, RiskDecision, RiskLevel,
    RiskScore,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MockFraudOutcome {
    Allow,
    Challenge,
    Review,
    Reject,
    ProviderError,
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
            MockFraudOutcome::ProviderError => Err(PaymentError::ProviderDetails {
                details: ProviderErrorDetails {
                    provider: PaymentProvider::other("mock-fraud"),
                    status: 503,
                    code: Some("mock_unavailable".to_owned()),
                    request_id: Some("mock_req_123".to_owned()),
                    message: "mock fraud provider unavailable".to_owned(),
                },
            }),
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

fn payment_request() -> CreatePaymentRequest {
    CreatePaymentRequest::builder()
        .amount(Money::new_minor(1_000, "USD").expect("money should be valid"))
        .reference("ORDER-1")
        .expect("reference should be valid")
        .payment_method(PaymentMethod::card())
        .build()
        .expect("request should be valid")
}

#[test]
fn mock_provider_returns_successful_outcomes() {
    let request = payment_request();
    let expectations = [
        (MockFraudOutcome::Allow, RiskDecision::Allow, RiskLevel::Low),
        (
            MockFraudOutcome::Challenge,
            RiskDecision::Challenge,
            RiskLevel::Medium,
        ),
        (
            MockFraudOutcome::Review,
            RiskDecision::Review,
            RiskLevel::High,
        ),
        (
            MockFraudOutcome::Reject,
            RiskDecision::Reject,
            RiskLevel::Critical,
        ),
    ];

    for (outcome, decision, level) in expectations {
        let assessment = MockFraudProvider::new(outcome)
            .assess(&request)
            .expect("mock assessment should succeed");

        assert_eq!(assessment.decision(), decision);
        assert_eq!(assessment.level(), Some(level));
        assert_eq!(
            assessment
                .provider_reference()
                .expect("provider reference should exist")
                .as_str(),
            "mock_assessment_123"
        );
    }
}

#[test]
fn mock_provider_error_is_redacted() {
    let request = payment_request();
    let error = MockFraudProvider::new(MockFraudOutcome::ProviderError)
        .assess(&request)
        .expect_err("mock provider error should fail");
    let display = error.to_string();

    assert!(matches!(error, PaymentError::ProviderDetails { .. }));
    assert!(display.contains("mock fraud provider unavailable"));
    assert!(!display.contains("ORDER-1"));
}
