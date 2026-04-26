use crate::PaymentError;

/// Validated E.164-style phone number.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhoneNumber(String);

impl PhoneNumber {
    /// Creates a phone number from digits with an optional leading `+`.
    ///
    /// # Errors
    ///
    /// Returns an error when the phone number is outside E.164 length bounds or
    /// contains unsupported characters.
    pub fn new(value: impl AsRef<str>) -> Result<Self, PaymentError> {
        let value = value.as_ref().trim();
        let digits = value.strip_prefix('+').unwrap_or(value);
        if !(8..=15).contains(&digits.len()) || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(PaymentError::InvalidPhoneNumber(value.to_owned()));
        }

        Ok(Self(format!("+{digits}")))
    }

    /// Creates a phone number without adding the display `+` prefix.
    ///
    /// # Errors
    ///
    /// Returns an error when the number is invalid.
    pub fn new_digits(value: impl AsRef<str>) -> Result<Self, PaymentError> {
        Self::new(value)
    }

    /// Returns the normalized number with a leading `+`.
    #[inline]
    #[must_use]
    pub fn as_e164(&self) -> &str {
        &self.0
    }

    /// Returns the normalized digits without the leading `+`.
    #[inline]
    #[must_use]
    pub fn digits(&self) -> &str {
        &self.0[1..]
    }
}

impl AsRef<str> for PhoneNumber {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_e164()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_normalizes_number() {
        let phone = PhoneNumber::new("260971234567").expect("phone should be valid");
        let phone_from_digits =
            PhoneNumber::new_digits("260971234567").expect("phone should be valid");

        assert_eq!(phone.as_e164(), "+260971234567");
        assert_eq!(phone.digits(), "260971234567");
        assert_eq!(phone_from_digits.as_ref(), "+260971234567");
    }

    #[test]
    fn new_rejects_short_number() {
        assert!(matches!(
            PhoneNumber::new("123"),
            Err(PaymentError::InvalidPhoneNumber(_))
        ));
    }
}
