use crate::PaymentError;

/// ISO 3166-1 alpha-2 country code.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CountryCode(String);

impl CountryCode {
    /// Parses and validates an ISO 3166-1 alpha-2 country code.
    ///
    /// # Errors
    ///
    /// Returns an error when the code is not exactly two ASCII letters.
    pub fn new(code: impl AsRef<str>) -> Result<Self, PaymentError> {
        let code = code.as_ref().trim();
        if code.len() != 2 || !code.bytes().all(|byte| byte.is_ascii_alphabetic()) {
            return Err(PaymentError::InvalidCountryCode(code.to_owned()));
        }

        Ok(Self(code.to_ascii_uppercase()))
    }

    /// Returns the normalized country code.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for CountryCode {
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
        let code = CountryCode::new("zm").expect("country should be valid");

        assert_eq!(code.as_str(), "ZM");
    }

    #[test]
    fn new_rejects_invalid_code() {
        assert!(matches!(
            CountryCode::new("ZMB"),
            Err(PaymentError::InvalidCountryCode(_))
        ));
    }
}
