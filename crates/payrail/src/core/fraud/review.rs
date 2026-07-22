use crate::FraudProviderReference;

/// Provider-neutral manual review request metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReviewRequest {
    provider_reference: Option<FraudProviderReference>,
    message: Option<String>,
}

impl ReviewRequest {
    /// Creates an empty review request.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the provider review reference.
    #[must_use]
    pub fn with_provider_reference(mut self, reference: FraudProviderReference) -> Self {
        self.provider_reference = Some(reference);
        self
    }

    /// Sets a safe, redacted review message.
    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Returns the provider review reference.
    #[inline]
    #[must_use]
    pub const fn provider_reference(&self) -> Option<&FraudProviderReference> {
        self.provider_reference.as_ref()
    }

    /// Returns the safe, redacted review message.
    #[inline]
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}
