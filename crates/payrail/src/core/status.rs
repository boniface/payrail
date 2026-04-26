/// Normalized payment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PaymentStatus {
    /// Created but not started.
    Created,
    /// Requires customer or merchant action.
    RequiresAction,
    /// Pending provider completion.
    Pending,
    /// Processing asynchronously.
    Processing,
    /// Authorized but not captured.
    Authorized,
    /// Successfully completed.
    Succeeded,
    /// Failed.
    Failed,
    /// Cancelled.
    Cancelled,
    /// Expired.
    Expired,
    /// Fully refunded.
    Refunded,
    /// Partially refunded.
    PartiallyRefunded,
}
