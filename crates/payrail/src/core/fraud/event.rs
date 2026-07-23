use crate::{
    FraudProvider, FraudProviderReference, MerchantReference, PaymentProvider, ProviderReference,
    RiskAssessment, WebhookEventId,
};

/// Normalized fraud event emitted by a payment or fraud provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FraudEvent {
    id: Option<WebhookEventId>,
    provider: FraudProvider,
    provider_reference: Option<FraudProviderReference>,
    payment_provider: Option<PaymentProvider>,
    payment_provider_reference: Option<ProviderReference>,
    merchant_reference: Option<MerchantReference>,
    event_type: FraudEventType,
    assessment: Option<RiskAssessment>,
    message: Option<String>,
}

impl FraudEvent {
    /// Creates a normalized fraud event.
    #[inline]
    #[must_use]
    pub const fn new(provider: FraudProvider, event_type: FraudEventType) -> Self {
        Self {
            id: None,
            provider,
            provider_reference: None,
            payment_provider: None,
            payment_provider_reference: None,
            merchant_reference: None,
            event_type,
            assessment: None,
            message: None,
        }
    }

    /// Sets the provider event ID.
    #[inline]
    #[must_use]
    pub fn with_id(mut self, id: WebhookEventId) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the fraud provider reference.
    #[inline]
    #[must_use]
    pub fn with_provider_reference(mut self, reference: FraudProviderReference) -> Self {
        self.provider_reference = Some(reference);
        self
    }

    /// Sets the related payment provider reference.
    #[inline]
    #[must_use]
    pub fn with_payment_reference(
        mut self,
        provider: PaymentProvider,
        reference: ProviderReference,
    ) -> Self {
        self.payment_provider = Some(provider);
        self.payment_provider_reference = Some(reference);
        self
    }

    /// Sets the merchant reference.
    #[inline]
    #[must_use]
    pub fn with_merchant_reference(mut self, reference: MerchantReference) -> Self {
        self.merchant_reference = Some(reference);
        self
    }

    /// Sets the normalized assessment.
    #[inline]
    #[must_use]
    pub fn with_assessment(mut self, assessment: RiskAssessment) -> Self {
        self.assessment = Some(assessment);
        self
    }

    /// Sets a safe provider message.
    #[inline]
    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Returns the provider event ID.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> Option<&WebhookEventId> {
        self.id.as_ref()
    }

    /// Returns the fraud provider.
    #[inline]
    #[must_use]
    pub const fn provider(&self) -> &FraudProvider {
        &self.provider
    }

    /// Returns the fraud provider reference.
    #[inline]
    #[must_use]
    pub const fn provider_reference(&self) -> Option<&FraudProviderReference> {
        self.provider_reference.as_ref()
    }

    /// Returns the related payment provider.
    #[inline]
    #[must_use]
    pub const fn payment_provider(&self) -> Option<&PaymentProvider> {
        self.payment_provider.as_ref()
    }

    /// Returns the related payment provider reference.
    #[inline]
    #[must_use]
    pub const fn payment_provider_reference(&self) -> Option<&ProviderReference> {
        self.payment_provider_reference.as_ref()
    }

    /// Returns the merchant reference.
    #[inline]
    #[must_use]
    pub const fn merchant_reference(&self) -> Option<&MerchantReference> {
        self.merchant_reference.as_ref()
    }

    /// Returns the normalized event type.
    #[inline]
    #[must_use]
    pub const fn event_type(&self) -> FraudEventType {
        self.event_type
    }

    /// Returns the normalized assessment.
    #[inline]
    #[must_use]
    pub const fn assessment(&self) -> Option<&RiskAssessment> {
        self.assessment.as_ref()
    }

    /// Returns the safe provider message.
    #[inline]
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

/// Normalized fraud event type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FraudEventType {
    /// A risk assessment was created.
    RiskAssessmentCreated,
    /// A risk assessment was updated.
    RiskAssessmentUpdated,
    /// A fraud warning was created.
    EarlyFraudWarningCreated,
    /// A provider review was opened.
    ReviewOpened,
    /// A provider review was approved.
    ReviewApproved,
    /// A provider review was rejected.
    ReviewRejected,
    /// A provider review expired.
    ReviewExpired,
    /// A dispute was opened.
    DisputeOpened,
    /// A dispute changed state.
    DisputeUpdated,
    /// A dispute was won.
    DisputeWon,
    /// A dispute was lost.
    DisputeLost,
}

#[cfg(test)]
mod tests {
    use crate::{PaymentProvider, RiskDecision};

    use super::*;

    #[test]
    fn fraud_event_accessors_return_configured_values() {
        let event = FraudEvent::new(FraudProvider::StripeRadar, FraudEventType::DisputeOpened)
            .with_id(WebhookEventId::new("evt_123").expect("event id should be valid"))
            .with_provider_reference(
                FraudProviderReference::new("du_123").expect("reference should be valid"),
            )
            .with_payment_reference(
                PaymentProvider::Stripe,
                ProviderReference::new("pi_123").expect("reference should be valid"),
            )
            .with_merchant_reference(
                MerchantReference::new("ORDER-1").expect("reference should be valid"),
            )
            .with_assessment(RiskAssessment::new(RiskDecision::Review))
            .with_message("dispute opened");

        assert_eq!(
            event.id().expect("event id should exist").as_str(),
            "evt_123"
        );
        assert_eq!(event.provider(), &FraudProvider::StripeRadar);
        assert_eq!(
            event
                .provider_reference()
                .expect("provider reference should exist")
                .as_str(),
            "du_123"
        );
        assert_eq!(event.payment_provider(), Some(&PaymentProvider::Stripe));
        assert_eq!(
            event
                .payment_provider_reference()
                .expect("payment reference should exist")
                .as_str(),
            "pi_123"
        );
        assert_eq!(
            event
                .merchant_reference()
                .expect("merchant reference should exist")
                .as_str(),
            "ORDER-1"
        );
        assert_eq!(event.event_type(), FraudEventType::DisputeOpened);
        assert_eq!(
            event
                .assessment()
                .expect("assessment should exist")
                .decision(),
            RiskDecision::Review
        );
        assert_eq!(event.message(), Some("dispute opened"));
    }
}
