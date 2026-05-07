use crate::{
    MerchantReference, Money, PaymentProvider, PaymentStatus, ProviderReference, WebhookEventId,
};

/// Normalized payment event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentEvent {
    /// Provider event ID.
    pub id: Option<WebhookEventId>,
    /// Provider that emitted the event.
    pub provider: PaymentProvider,
    /// Provider payment reference.
    pub provider_reference: ProviderReference,
    /// Optional merchant reference.
    pub merchant_reference: Option<MerchantReference>,
    /// Normalized status.
    pub status: PaymentStatus,
    /// Optional amount.
    pub amount: Option<Money>,
    /// Normalized event type.
    pub event_type: PaymentEventType,
    /// Safe provider message.
    pub message: Option<String>,
}

impl PaymentEvent {
    /// Returns the provider event ID.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> Option<&WebhookEventId> {
        self.id.as_ref()
    }

    /// Returns the provider that emitted the event.
    #[inline]
    #[must_use]
    pub const fn provider(&self) -> &PaymentProvider {
        &self.provider
    }

    /// Returns the provider payment reference.
    #[inline]
    #[must_use]
    pub const fn provider_reference(&self) -> &ProviderReference {
        &self.provider_reference
    }

    /// Returns the merchant reference when present.
    #[inline]
    #[must_use]
    pub const fn merchant_reference(&self) -> Option<&MerchantReference> {
        self.merchant_reference.as_ref()
    }

    /// Returns the normalized payment status.
    #[inline]
    #[must_use]
    pub const fn status(&self) -> PaymentStatus {
        self.status
    }

    /// Returns the amount when present.
    #[inline]
    #[must_use]
    pub const fn amount(&self) -> Option<&Money> {
        self.amount.as_ref()
    }

    /// Returns the normalized event type.
    #[inline]
    #[must_use]
    pub const fn event_type(&self) -> PaymentEventType {
        self.event_type
    }

    /// Returns the safe provider message.
    #[inline]
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

/// Normalized payment event type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PaymentEventType {
    /// Payment was created.
    PaymentCreated,
    /// Payment requires action.
    PaymentRequiresAction,
    /// Payment is pending.
    PaymentPending,
    /// Payment succeeded.
    PaymentSucceeded,
    /// Payment failed.
    PaymentFailed,
    /// Payment was cancelled.
    PaymentCancelled,
    /// Payment was refunded.
    PaymentRefunded,
    /// Refund was created.
    RefundCreated,
    /// Refund failed.
    RefundFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payment_event_accessors_return_normalized_fields() {
        let amount = Money::new_minor(1_000, "USD").expect("money should be valid");
        let event = PaymentEvent {
            id: Some(WebhookEventId::new("evt_123").expect("event id should be valid")),
            provider: PaymentProvider::Stripe,
            provider_reference: ProviderReference::new("pi_123")
                .expect("reference should be valid"),
            merchant_reference: Some(
                MerchantReference::new("ORDER-1").expect("reference should be valid"),
            ),
            status: PaymentStatus::Succeeded,
            amount: Some(amount.clone()),
            event_type: PaymentEventType::PaymentSucceeded,
            message: Some("payment succeeded".to_owned()),
        };

        assert_eq!(
            event.id().expect("event id should be present").as_str(),
            "evt_123"
        );
        assert_eq!(event.provider(), &PaymentProvider::Stripe);
        assert_eq!(event.provider_reference().as_str(), "pi_123");
        assert_eq!(
            event
                .merchant_reference()
                .expect("merchant reference should be present")
                .as_str(),
            "ORDER-1"
        );
        assert_eq!(event.status(), PaymentStatus::Succeeded);
        assert_eq!(event.amount(), Some(&amount));
        assert_eq!(event.event_type(), PaymentEventType::PaymentSucceeded);
        assert_eq!(event.message(), Some("payment succeeded"));
    }

    #[test]
    fn payment_event_accessors_handle_absent_optional_fields() {
        let event = PaymentEvent {
            id: None,
            provider: PaymentProvider::PayPal,
            provider_reference: ProviderReference::new("ORDER-1")
                .expect("reference should be valid"),
            merchant_reference: None,
            status: PaymentStatus::Pending,
            amount: None,
            event_type: PaymentEventType::PaymentPending,
            message: None,
        };

        assert!(event.id().is_none());
        assert!(event.merchant_reference().is_none());
        assert!(event.amount().is_none());
        assert_eq!(event.message(), None);
    }
}
