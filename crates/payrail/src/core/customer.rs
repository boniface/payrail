use crate::{CountryCode, PhoneNumber};

/// Optional customer details attached to a payment request.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Customer {
    email: Option<String>,
    phone: Option<PhoneNumber>,
    country: Option<CountryCode>,
    name: Option<String>,
}

impl Customer {
    /// Creates an empty customer.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the email address.
    #[must_use]
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Sets the phone number.
    #[must_use]
    pub fn with_phone(mut self, phone: PhoneNumber) -> Self {
        self.phone = Some(phone);
        self
    }

    /// Sets the country.
    #[must_use]
    pub fn with_country(mut self, country: CountryCode) -> Self {
        self.country = Some(country);
        self
    }

    /// Sets the customer name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Returns the email address.
    #[inline]
    #[must_use]
    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }

    /// Returns the phone number.
    #[inline]
    #[must_use]
    pub const fn phone(&self) -> Option<&PhoneNumber> {
        self.phone.as_ref()
    }

    /// Returns the country.
    #[inline]
    #[must_use]
    pub const fn country(&self) -> Option<&CountryCode> {
        self.country.as_ref()
    }

    /// Returns the customer name.
    #[inline]
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_methods_set_customer_fields() {
        let phone = PhoneNumber::new("260971234567").expect("phone should be valid");
        let country = CountryCode::new("ZM").expect("country should be valid");

        let customer = Customer::new()
            .with_email("customer@example.com")
            .with_phone(phone)
            .with_country(country)
            .with_name("Ada Buyer");

        assert_eq!(customer.email(), Some("customer@example.com"));
        assert_eq!(
            customer.phone().expect("phone should be present").as_e164(),
            "+260971234567"
        );
        assert_eq!(
            customer
                .country()
                .expect("country should be present")
                .as_str(),
            "ZM"
        );
        assert_eq!(customer.name(), Some("Ada Buyer"));
    }
}
