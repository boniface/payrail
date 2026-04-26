use crate::PaymentError;

/// Idempotency key used to make provider write operations retry-safe.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    /// Creates a validated idempotency key.
    ///
    /// # Errors
    ///
    /// Returns an error when the key is empty or longer than 255 bytes.
    pub fn new(value: impl AsRef<str>) -> Result<Self, PaymentError> {
        let value = value.as_ref().trim();
        if value.is_empty() || value.len() > 255 {
            return Err(PaymentError::InvalidIdempotencyKey(value.to_owned()));
        }

        Ok(Self(value.to_owned()))
    }

    /// Returns the idempotency key.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for IdempotencyKey {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_key() {
        let key = IdempotencyKey::new("ORDER-123:create").expect("key should be valid");

        assert_eq!(key.as_str(), "ORDER-123:create");
    }

    #[test]
    fn new_rejects_empty_key() {
        assert!(matches!(
            IdempotencyKey::new(""),
            Err(PaymentError::InvalidIdempotencyKey(_))
        ));
    }
}
