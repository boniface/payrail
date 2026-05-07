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
