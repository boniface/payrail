use crate::PaymentError;

/// Normalized fraud risk score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RiskScore(u16);

impl RiskScore {
    /// Highest valid normalized fraud risk score.
    pub const MAX: u16 = 1_000;

    /// Creates a normalized fraud risk score.
    ///
    /// # Errors
    ///
    /// Returns an error when the score is greater than `1000`.
    #[inline]
    pub const fn new(value: u16) -> Result<Self, PaymentError> {
        if value > Self::MAX {
            return Err(PaymentError::InvalidRiskScore(value));
        }

        Ok(Self(value))
    }

    /// Returns the raw normalized score.
    #[inline]
    #[must_use]
    pub const fn value(self) -> u16 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_scores() {
        assert_eq!(RiskScore::new(0).expect("score should be valid").value(), 0);
        assert_eq!(
            RiskScore::new(500).expect("score should be valid").value(),
            500
        );
        assert_eq!(
            RiskScore::new(1_000)
                .expect("score should be valid")
                .value(),
            1_000
        );
    }

    #[test]
    fn rejects_scores_above_maximum() {
        assert!(matches!(
            RiskScore::new(1_001),
            Err(PaymentError::InvalidRiskScore(1_001))
        ));
    }
}
