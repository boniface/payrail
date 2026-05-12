use crate::PaymentError;

/// Integer amount in the smallest currency unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MinorAmount(i64);

impl MinorAmount {
    /// Creates a positive minor-unit amount.
    ///
    /// # Errors
    ///
    /// Returns an error when `value` is zero or negative.
    #[inline]
    pub const fn new(value: i64) -> Result<Self, PaymentError> {
        if value <= 0 {
            return Err(PaymentError::InvalidAmount(value));
        }

        Ok(Self(value))
    }

    /// Returns the raw minor-unit value.
    #[inline]
    #[must_use]
    pub const fn value(self) -> i64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_positive_amount() {
        let amount = MinorAmount::new(100).expect("positive amount should be valid");

        assert_eq!(amount.value(), 100);
    }

    #[test]
    fn new_rejects_zero_amount() {
        assert!(matches!(
            MinorAmount::new(0),
            Err(PaymentError::InvalidAmount(0))
        ));
    }
}
