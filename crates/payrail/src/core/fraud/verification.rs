/// Verification status for customer, account, and identity signals.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum VerificationStatus {
    /// Verification was not provided.
    #[default]
    NotProvided,
    /// Verification is pending.
    Pending,
    /// Verification succeeded.
    Verified,
    /// Verification failed.
    Failed,
    /// Verification expired.
    Expired,
}
