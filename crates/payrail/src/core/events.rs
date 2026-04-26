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
