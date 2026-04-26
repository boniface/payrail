use crate::{CurrencyCode, MinorAmount, PaymentError};

/// Money represented in minor units.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Money {
    amount: MinorAmount,
    currency: CurrencyCode,
}

impl Money {
    /// Creates money from a minor-unit amount and currency code.
    ///
    /// # Errors
    ///
    /// Returns an error when either component is invalid.
    pub fn new_minor(amount: i64, currency: impl AsRef<str>) -> Result<Self, PaymentError> {
        Ok(Self {
            amount: MinorAmount::new(amount)?,
            currency: CurrencyCode::new(currency)?,
        })
    }

    /// Returns the amount.
    #[inline]
    #[must_use]
    pub const fn amount(&self) -> MinorAmount {
        self.amount
    }

    /// Returns the currency.
    #[inline]
    #[must_use]
    pub const fn currency(&self) -> &CurrencyCode {
        &self.currency
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_minor_builds_money() {
        let money = Money::new_minor(1_000, "usd").expect("money should be valid");

        assert_eq!(money.amount().value(), 1_000);
        assert_eq!(money.currency().as_str(), "USD");
    }
}
