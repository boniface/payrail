use crate::{PaymentError, PhoneNumber};

/// Normalizes a Zambia phone number to PayRail's E.164 representation.
///
/// # Errors
///
/// Returns an error when the number cannot be normalized to a Zambia number.
pub fn normalize_zambia_phone(value: impl AsRef<str>) -> Result<PhoneNumber, PaymentError> {
    let value = value.as_ref().trim();
    let digits = value.strip_prefix('+').unwrap_or(value);
    let normalized = if digits.starts_with("260") {
        digits.to_owned()
    } else if digits.starts_with('0') && digits.len() == 10 {
        format!("260{}", &digits[1..])
    } else if digits.len() == 9 {
        format!("260{digits}")
    } else {
        return Err(PaymentError::InvalidPhoneNumber(value.to_owned()));
    };

    PhoneNumber::new(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_accepts_local_number() {
        let phone = normalize_zambia_phone("0971234567").expect("phone should normalize");

        assert_eq!(phone.as_e164(), "+260971234567");
    }

    #[test]
    fn normalize_rejects_invalid_number() {
        assert!(matches!(
            normalize_zambia_phone("123"),
            Err(PaymentError::InvalidPhoneNumber(_))
        ));
    }
}
