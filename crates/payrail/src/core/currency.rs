use crate::PaymentError;

/// ISO 4217 currency code.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CurrencyCode(String);

impl CurrencyCode {
    /// Parses and validates an ISO 4217-style currency code.
    ///
    /// # Errors
    ///
    /// Returns an error when the code is not exactly three ASCII letters.
    pub fn new(code: impl AsRef<str>) -> Result<Self, PaymentError> {
        let code = code.as_ref().trim();
        if code.len() != 3 || !code.bytes().all(|byte| byte.is_ascii_alphabetic()) {
            return Err(PaymentError::InvalidCurrencyCode(code.to_owned()));
        }

        Ok(Self(code.to_ascii_uppercase()))
    }

    /// Returns the normalized currency code.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the ISO 4217 minor-unit exponent used by common providers.
    #[inline]
    #[must_use]
    pub fn minor_unit_exponent(&self) -> u8 {
        match self.0.as_str() {
            "BIF" | "CLP" | "DJF" | "GNF" | "JPY" | "KMF" | "KRW" | "MGA" | "PYG" | "RWF"
            | "UGX" | "VND" | "VUV" | "XAF" | "XOF" | "XPF" => 0,
            "BHD" | "IQD" | "JOD" | "KWD" | "LYD" | "OMR" | "TND" => 3,
            "CLF" | "UYW" => 4,
            _ => 2,
        }
    }

    /// Returns the multiplier between major and minor units.
    #[inline]
    #[must_use]
    pub fn minor_unit_factor(&self) -> i64 {
        10_i64.pow(u32::from(self.minor_unit_exponent()))
    }

    /// Formats a minor-unit amount as a provider decimal amount string.
    #[must_use]
    pub fn format_minor_units(&self, minor: i64) -> String {
        let exponent = self.minor_unit_exponent();
        if exponent == 0 {
            return minor.to_string();
        }

        let factor = self.minor_unit_factor().cast_unsigned();
        let sign = if minor < 0 { "-" } else { "" };
        let absolute = minor.unsigned_abs();
        let major = absolute / factor;
        let fractional = absolute % factor;
        let width = usize::from(exponent);
        format!("{sign}{major}.{fractional:0width$}")
    }

    /// Converts a minor-unit amount into an integer major-unit amount.
    ///
    /// # Errors
    ///
    /// Returns an error when the minor amount cannot be represented as a whole major unit.
    pub fn minor_units_to_major_integer(&self, minor: i64) -> Result<i64, PaymentError> {
        let factor = self.minor_unit_factor();
        if minor % factor != 0 {
            return Err(PaymentError::InvalidAmount(minor));
        }

        Ok(minor / factor)
    }

    /// Converts an integer major-unit amount into minor units.
    ///
    /// # Errors
    ///
    /// Returns an error when the multiplication overflows.
    pub fn major_integer_to_minor_units(&self, major: i64) -> Result<i64, PaymentError> {
        major
            .checked_mul(self.minor_unit_factor())
            .ok_or(PaymentError::InvalidAmount(major))
    }
}

impl AsRef<str> for CurrencyCode {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_uppercases_valid_code() {
        let code = CurrencyCode::new("usd").expect("currency should be valid");

        assert_eq!(code.as_str(), "USD");
    }

    #[test]
    fn new_rejects_invalid_code() {
        assert!(matches!(
            CurrencyCode::new("US1"),
            Err(PaymentError::InvalidCurrencyCode(_))
        ));
    }

    #[test]
    fn formats_minor_units_using_currency_exponent() {
        let usd = CurrencyCode::new("USD").expect("currency should be valid");
        let jpy = CurrencyCode::new("JPY").expect("currency should be valid");
        let kwd = CurrencyCode::new("KWD").expect("currency should be valid");

        assert_eq!(usd.format_minor_units(1234), "12.34");
        assert_eq!(usd.format_minor_units(-1234), "-12.34");
        assert_eq!(jpy.format_minor_units(1234), "1234");
        assert_eq!(kwd.format_minor_units(1234), "1.234");
    }

    #[test]
    fn converts_major_and_minor_units_without_truncation() {
        let zmw = CurrencyCode::new("ZMW").expect("currency should be valid");

        assert_eq!(
            zmw.minor_units_to_major_integer(5_000)
                .expect("minor amount should convert"),
            50
        );
        assert!(matches!(
            zmw.minor_units_to_major_integer(5_050),
            Err(PaymentError::InvalidAmount(5_050))
        ));
        assert_eq!(
            zmw.major_integer_to_minor_units(50)
                .expect("major amount should convert"),
            5_000
        );
    }
}
