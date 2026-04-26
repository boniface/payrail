use crate::{CountryCode, CurrencyCode, PaymentProvider};

/// Redacted provider error details safe to expose to application code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderErrorDetails {
    /// Provider that returned the error.
    pub provider: PaymentProvider,
    /// HTTP status code.
    pub status: u16,
    /// Safe provider error code.
    pub code: Option<String>,
    /// Safe provider request ID.
    pub request_id: Option<String>,
    /// Redacted message.
    pub message: String,
}

/// PayRail error type.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PaymentError {
    /// Invalid positive amount.
    #[error("invalid amount: {0}")]
    InvalidAmount(i64),

    /// Invalid currency code.
    #[error("invalid currency code: {0}")]
    InvalidCurrencyCode(String),

    /// Invalid country code.
    #[error("invalid country code: {0}")]
    InvalidCountryCode(String),

    /// Invalid reference.
    #[error("invalid reference: {0}")]
    InvalidReference(String),

    /// Invalid idempotency key.
    #[error("invalid idempotency key: {0}")]
    InvalidIdempotencyKey(String),

    /// Invalid phone number.
    #[error("invalid phone number")]
    InvalidPhoneNumber(String),

    /// Invalid URL.
    #[error("invalid url: {0}")]
    InvalidUrl(String),

    /// A required field was missing.
    #[error("missing required field: {0}")]
    MissingRequiredField(&'static str),

    /// Invalid configuration.
    #[error("invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Connector is not configured.
    #[error("connector not configured: {provider:?}")]
    ConnectorNotConfigured { provider: PaymentProvider },

    /// Unsupported payment method.
    #[error("unsupported payment method: {0}")]
    UnsupportedPaymentMethod(String),

    /// Unsupported country.
    #[error("unsupported country: {0:?}")]
    UnsupportedCountry(CountryCode),

    /// Unsupported currency.
    #[error("unsupported currency: {0:?}")]
    UnsupportedCurrency(CurrencyCode),

    /// Unsupported route.
    #[error("unsupported payment route: method={method}, country={country:?}")]
    UnsupportedPaymentRoute {
        /// Method description.
        method: String,
        /// Optional country.
        country: Option<CountryCode>,
    },

    /// Provider authentication failed.
    #[error("provider authentication failed")]
    AuthenticationFailed,

    /// Provider request failed.
    #[error("provider request failed: {provider:?}, status={status}, message={message}")]
    ProviderRequestFailed {
        /// Provider.
        provider: PaymentProvider,
        /// HTTP status.
        status: u16,
        /// Redacted message.
        message: String,
    },

    /// Redacted provider details.
    #[error("provider request failed: {details:?}")]
    ProviderDetails {
        /// Details.
        details: ProviderErrorDetails,
    },

    /// Provider unavailable.
    #[error("provider unavailable: {0:?}")]
    ProviderUnavailable(PaymentProvider),

    /// Provider rate limited.
    #[error("rate limited by provider: {0:?}")]
    RateLimited(PaymentProvider),

    /// Webhook verification failed.
    #[error("webhook verification failed")]
    WebhookVerificationFailed,

    /// Invalid webhook payload.
    #[error("webhook payload invalid: {0}")]
    InvalidWebhookPayload(String),

    /// Unsupported operation.
    #[error("operation not supported: {0}")]
    UnsupportedOperation(String),

    /// HTTP error.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_phone_error_does_not_display_value() {
        let error = PaymentError::InvalidPhoneNumber("+260971234567".to_owned());

        assert_eq!(error.to_string(), "invalid phone number");
    }
}
