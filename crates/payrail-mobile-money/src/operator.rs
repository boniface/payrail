use payrail_core::MobileMoneyOperator;

/// Supported Zambia Mobile Money operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZambiaOperator {
    /// MTN Mobile Money.
    Mtn,
    /// Airtel Money.
    Airtel,
    /// Zamtel Kwacha.
    Zamtel,
}

impl From<ZambiaOperator> for MobileMoneyOperator {
    fn from(value: ZambiaOperator) -> Self {
        match value {
            ZambiaOperator::Mtn => Self::Mtn,
            ZambiaOperator::Airtel => Self::Airtel,
            ZambiaOperator::Zamtel => Self::Zamtel,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zambia_operator_converts_to_core_operator() {
        assert_eq!(
            MobileMoneyOperator::from(ZambiaOperator::Mtn),
            MobileMoneyOperator::Mtn
        );
        assert_eq!(
            MobileMoneyOperator::from(ZambiaOperator::Airtel),
            MobileMoneyOperator::Airtel
        );
        assert_eq!(
            MobileMoneyOperator::from(ZambiaOperator::Zamtel),
            MobileMoneyOperator::Zamtel
        );
    }
}
