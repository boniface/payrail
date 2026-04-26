use std::collections::BTreeMap;

use url::Url;
use uuid::Uuid;

use crate::{
    Customer, IdempotencyKey, MerchantReference, Money, NextAction, PaymentError, PaymentId,
    PaymentMethod, PaymentProvider, PaymentStatus, ProviderReference,
};

/// Request metadata.
pub type Metadata = BTreeMap<String, String>;

/// Request to create a payment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatePaymentRequest {
    amount: Money,
    reference: MerchantReference,
    description: Option<String>,
    customer: Option<Customer>,
    payment_method: PaymentMethod,
    callback_url: Option<Url>,
    return_url: Option<Url>,
    cancel_url: Option<Url>,
    idempotency_key: Option<IdempotencyKey>,
    metadata: Metadata,
}

impl CreatePaymentRequest {
    /// Starts a create payment request builder.
    #[inline]
    pub fn builder() -> CreatePaymentRequestBuilder {
        CreatePaymentRequestBuilder::default()
    }

    /// Returns the amount.
    #[inline]
    #[must_use]
    pub const fn amount(&self) -> &Money {
        &self.amount
    }

    /// Returns the merchant reference.
    #[inline]
    #[must_use]
    pub const fn reference(&self) -> &MerchantReference {
        &self.reference
    }

    /// Returns the description.
    #[inline]
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the customer.
    #[inline]
    #[must_use]
    pub const fn customer(&self) -> Option<&Customer> {
        self.customer.as_ref()
    }

    /// Returns the payment method.
    #[inline]
    #[must_use]
    pub const fn payment_method(&self) -> &PaymentMethod {
        &self.payment_method
    }

    /// Returns the callback URL.
    #[inline]
    #[must_use]
    pub const fn callback_url(&self) -> Option<&Url> {
        self.callback_url.as_ref()
    }

    /// Returns the return URL.
    #[inline]
    #[must_use]
    pub const fn return_url(&self) -> Option<&Url> {
        self.return_url.as_ref()
    }

    /// Returns the cancel URL.
    #[inline]
    #[must_use]
    pub const fn cancel_url(&self) -> Option<&Url> {
        self.cancel_url.as_ref()
    }

    /// Returns the idempotency key.
    #[inline]
    #[must_use]
    pub const fn idempotency_key(&self) -> Option<&IdempotencyKey> {
        self.idempotency_key.as_ref()
    }

    /// Returns metadata.
    #[inline]
    #[must_use]
    pub const fn metadata(&self) -> &Metadata {
        &self.metadata
    }
}

/// Builder for [`CreatePaymentRequest`].
#[derive(Debug, Default, Clone)]
#[must_use]
pub struct CreatePaymentRequestBuilder {
    amount: Option<Money>,
    reference: Option<MerchantReference>,
    description: Option<String>,
    customer: Option<Customer>,
    payment_method: Option<PaymentMethod>,
    callback_url: Option<Url>,
    return_url: Option<Url>,
    cancel_url: Option<Url>,
    idempotency_key: Option<IdempotencyKey>,
    metadata: Metadata,
}

impl CreatePaymentRequestBuilder {
    /// Sets the amount.
    pub fn amount(mut self, amount: Money) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Sets the merchant reference.
    ///
    /// # Errors
    ///
    /// Returns an error when the reference is invalid.
    pub fn reference(mut self, reference: impl AsRef<str>) -> Result<Self, PaymentError> {
        self.reference = Some(MerchantReference::new(reference)?);
        Ok(self)
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the customer.
    pub fn customer(mut self, customer: Customer) -> Self {
        self.customer = Some(customer);
        self
    }

    /// Sets the payment method.
    pub fn payment_method(mut self, payment_method: PaymentMethod) -> Self {
        self.payment_method = Some(payment_method);
        self
    }

    /// Sets the callback URL.
    ///
    /// # Errors
    ///
    /// Returns an error when the URL is invalid.
    pub fn callback_url(mut self, url: impl AsRef<str>) -> Result<Self, PaymentError> {
        self.callback_url = Some(parse_url(url.as_ref())?);
        Ok(self)
    }

    /// Sets the return URL.
    ///
    /// # Errors
    ///
    /// Returns an error when the URL is invalid.
    pub fn return_url(mut self, url: impl AsRef<str>) -> Result<Self, PaymentError> {
        self.return_url = Some(parse_url(url.as_ref())?);
        Ok(self)
    }

    /// Sets the cancel URL.
    ///
    /// # Errors
    ///
    /// Returns an error when the URL is invalid.
    pub fn cancel_url(mut self, url: impl AsRef<str>) -> Result<Self, PaymentError> {
        self.cancel_url = Some(parse_url(url.as_ref())?);
        Ok(self)
    }

    /// Sets the idempotency key.
    ///
    /// # Errors
    ///
    /// Returns an error when the key is invalid.
    pub fn idempotency_key(mut self, key: impl AsRef<str>) -> Result<Self, PaymentError> {
        self.idempotency_key = Some(IdempotencyKey::new(key)?);
        Ok(self)
    }

    /// Adds one metadata entry.
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Builds the request.
    ///
    /// # Errors
    ///
    /// Returns an error when a required field is missing.
    pub fn build(self) -> Result<CreatePaymentRequest, PaymentError> {
        Ok(CreatePaymentRequest {
            amount: self
                .amount
                .ok_or(PaymentError::MissingRequiredField("amount"))?,
            reference: self
                .reference
                .ok_or(PaymentError::MissingRequiredField("reference"))?,
            description: self.description,
            customer: self.customer,
            payment_method: self
                .payment_method
                .ok_or(PaymentError::MissingRequiredField("payment_method"))?,
            callback_url: self.callback_url,
            return_url: self.return_url,
            cancel_url: self.cancel_url,
            idempotency_key: self.idempotency_key,
            metadata: self.metadata,
        })
    }
}

fn parse_url(value: &str) -> Result<Url, PaymentError> {
    Url::parse(value).map_err(|error| PaymentError::InvalidUrl(error.to_string()))
}

/// Normalized payment creation response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentSession {
    /// PayRail payment ID.
    pub payment_id: PaymentId,
    /// Provider handling the payment.
    pub provider: PaymentProvider,
    /// Provider reference.
    pub provider_reference: ProviderReference,
    /// Merchant reference.
    pub merchant_reference: MerchantReference,
    /// Normalized status.
    pub status: PaymentStatus,
    /// Next required action.
    pub next_action: Option<NextAction>,
}

impl PaymentSession {
    /// Creates a normalized session with a generated PayRail payment ID.
    ///
    /// # Errors
    ///
    /// Returns an error if generated ID validation fails.
    pub fn new(
        provider: PaymentProvider,
        provider_reference: ProviderReference,
        merchant_reference: MerchantReference,
        status: PaymentStatus,
        next_action: Option<NextAction>,
    ) -> Result<Self, PaymentError> {
        Ok(Self {
            payment_id: PaymentId::new(format!("pay_{}", Uuid::new_v4()))?,
            provider,
            provider_reference,
            merchant_reference,
            status,
            next_action,
        })
    }
}

/// Normalized payment status response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentStatusResponse {
    /// Provider handling the payment.
    pub provider: PaymentProvider,
    /// Provider reference.
    pub provider_reference: ProviderReference,
    /// Normalized status.
    pub status: PaymentStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_requires_fields() {
        assert!(matches!(
            CreatePaymentRequest::builder().build(),
            Err(PaymentError::MissingRequiredField("amount"))
        ));
    }

    #[test]
    fn builder_creates_request() {
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(1_000, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .description("Order 1")
            .customer(Customer::new().with_email("customer@example.com"))
            .payment_method(PaymentMethod::card())
            .callback_url("https://example.com/webhook")
            .expect("url should be valid")
            .return_url("https://example.com/success")
            .expect("url should be valid")
            .cancel_url("https://example.com/cancel")
            .expect("url should be valid")
            .idempotency_key("ORDER-1:create")
            .expect("key should be valid")
            .metadata("cart", "primary")
            .build()
            .expect("request should be valid");

        assert_eq!(request.reference().as_str(), "ORDER-1");
        assert_eq!(request.description(), Some("Order 1"));
        assert_eq!(
            request
                .customer()
                .expect("customer should be present")
                .email(),
            Some("customer@example.com")
        );
        assert!(request.callback_url().is_some());
        assert!(request.return_url().is_some());
        assert!(request.cancel_url().is_some());
        assert_eq!(
            request
                .idempotency_key()
                .expect("key should be present")
                .as_str(),
            "ORDER-1:create"
        );
        assert_eq!(
            request
                .metadata()
                .get("cart")
                .expect("metadata should exist"),
            "primary"
        );
    }

    #[test]
    fn payment_session_new_generates_payment_id() {
        let session = PaymentSession::new(
            PaymentProvider::Stripe,
            ProviderReference::new("provider-ref").expect("reference should be valid"),
            MerchantReference::new("ORDER-1").expect("reference should be valid"),
            PaymentStatus::Created,
            None,
        )
        .expect("session should be valid");

        assert!(session.payment_id.as_str().starts_with("pay_"));
        assert_eq!(session.provider, PaymentProvider::Stripe);
    }
}
