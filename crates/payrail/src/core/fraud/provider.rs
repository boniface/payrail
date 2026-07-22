use crate::PaymentError;

/// Fraud provider metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FraudProvider {
    /// Stripe Radar.
    StripeRadar,
    /// PayPal fraud protection.
    PayPalFraudProtection,
    /// Sift.
    Sift,
    /// SEON.
    Seon,
    /// Riskified.
    Riskified,
    /// Forter.
    Forter,
    /// Onfido.
    Onfido,
    /// Sumsub.
    Sumsub,
    /// Provider metadata not modeled directly.
    Other(String),
}

impl FraudProvider {
    /// Creates provider metadata for a fraud provider not modeled directly.
    ///
    /// This does not make the provider routable.
    #[inline]
    #[must_use]
    pub fn other(provider: impl Into<String>) -> Self {
        Self::Other(provider.into())
    }
}

/// Fraud provider reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FraudProviderReference(String);

impl FraudProviderReference {
    /// Creates a validated fraud provider reference.
    ///
    /// # Errors
    ///
    /// Returns an error when the reference is empty or longer than 255 bytes.
    pub fn new(value: impl AsRef<str>) -> Result<Self, PaymentError> {
        let value = value.as_ref().trim();
        if value.is_empty() || value.len() > 255 {
            return Err(PaymentError::InvalidReference(value.to_owned()));
        }

        Ok(Self(value.to_owned()))
    }

    /// Returns the validated reference.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for FraudProviderReference {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn other_provider_is_metadata_only() {
        assert_eq!(
            FraudProvider::other("example"),
            FraudProvider::Other("example".to_owned())
        );
    }

    #[test]
    fn fraud_provider_reference_validates_value() {
        let reference =
            FraudProviderReference::new("review_123").expect("reference should be valid");

        assert_eq!(reference.as_str(), "review_123");
        assert!(matches!(
            FraudProviderReference::new(" "),
            Err(PaymentError::InvalidReference(_))
        ));
    }
}
