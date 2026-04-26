use crate::PaymentError;

macro_rules! string_newtype {
    ($name:ident, $error:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(String);

        impl $name {
            /// Creates a validated string newtype.
            ///
            /// # Errors
            ///
            /// Returns an error when the value is empty or longer than 255 bytes.
            pub fn new(value: impl AsRef<str>) -> Result<Self, PaymentError> {
                let value = value.as_ref().trim();
                if value.is_empty() || value.len() > 255 {
                    return Err(PaymentError::$error(value.to_owned()));
                }

                Ok(Self(value.to_owned()))
            }

            /// Returns the validated string.
            #[inline]
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Converts into the owned string.
            #[inline]
            #[must_use]
            pub fn into_string(self) -> String {
                self.0
            }
        }

        impl AsRef<str> for $name {
            #[inline]
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }
    };
}

string_newtype!(MerchantReference, InvalidReference, "Merchant reference.");
string_newtype!(ProviderReference, InvalidReference, "Provider reference.");
string_newtype!(PaymentId, InvalidReference, "PayRail payment identifier.");
string_newtype!(
    WebhookEventId,
    InvalidReference,
    "Webhook event identifier."
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_reference() {
        let reference = MerchantReference::new("ORDER-123").expect("reference should be valid");

        assert_eq!(reference.as_str(), "ORDER-123");
        assert_eq!(reference.into_string(), "ORDER-123");
    }

    #[test]
    fn new_rejects_empty_reference() {
        assert!(matches!(
            ProviderReference::new(" "),
            Err(PaymentError::InvalidReference(_))
        ));
    }
}
