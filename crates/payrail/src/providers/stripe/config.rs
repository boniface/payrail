use std::time::Duration;

use crate::PaymentError;
use secrecy::{ExposeSecret, SecretString};
use url::Url;

const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Stripe connector configuration.
#[derive(Debug, Clone)]
pub struct StripeConfig {
    secret_key: SecretString,
    webhook_secret: Option<SecretString>,
    api_base: Url,
    request_timeout: Duration,
}

impl StripeConfig {
    /// Creates Stripe config using the default API base.
    ///
    /// # Errors
    ///
    /// Returns an error when the API base cannot be parsed.
    pub fn new(secret_key: SecretString) -> Result<Self, PaymentError> {
        validate_secret_key(&secret_key)?;
        Ok(Self {
            secret_key,
            webhook_secret: None,
            api_base: Url::parse("https://api.stripe.com")
                .map_err(|error| PaymentError::InvalidConfiguration(error.to_string()))?,
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
        })
    }

    /// Sets the webhook secret.
    #[must_use]
    pub fn webhook_secret(mut self, secret: Option<SecretString>) -> Self {
        self.webhook_secret = secret;
        self
    }

    /// Sets a custom API base URL.
    #[must_use]
    pub fn api_base(mut self, api_base: Url) -> Self {
        self.api_base = api_base;
        self
    }

    /// Sets the request timeout.
    ///
    /// # Errors
    ///
    /// Returns an error when the timeout is zero.
    pub fn request_timeout(mut self, timeout: Duration) -> Result<Self, PaymentError> {
        if timeout.is_zero() {
            return Err(PaymentError::InvalidConfiguration(
                "stripe request timeout cannot be zero".to_owned(),
            ));
        }

        self.request_timeout = timeout;
        Ok(self)
    }

    /// Returns the secret key.
    #[inline]
    #[must_use]
    pub const fn secret_key(&self) -> &SecretString {
        &self.secret_key
    }

    /// Returns the webhook secret.
    #[inline]
    #[must_use]
    pub const fn webhook_secret_value(&self) -> Option<&SecretString> {
        self.webhook_secret.as_ref()
    }

    /// Returns the API base.
    #[inline]
    #[must_use]
    pub const fn api_base_url(&self) -> &Url {
        &self.api_base
    }

    pub(crate) const fn request_timeout_value(&self) -> Duration {
        self.request_timeout
    }
}

fn validate_secret_key(secret_key: &SecretString) -> Result<(), PaymentError> {
    let secret = secret_key.expose_secret();
    if secret.starts_with("sk_live_") {
        reject_live_mode("stripe live secret keys")?;
        return Ok(());
    }
    if secret.starts_with("sk_test_") {
        return Ok(());
    }

    Err(PaymentError::InvalidConfiguration(
        "stripe secret key must start with sk_test_ or sk_live_".to_owned(),
    ))
}

fn reject_live_mode(label: &str) -> Result<(), PaymentError> {
    if cfg!(debug_assertions) && !live_mode_allowed() {
        return Err(PaymentError::InvalidConfiguration(format!(
            "{label} require PAYRAIL_ALLOW_LIVE_TESTS=true in debug or test builds"
        )));
    }

    Ok(())
}

fn live_mode_allowed() -> bool {
    std::env::var("PAYRAIL_ALLOW_LIVE_TESTS").is_ok_and(|value| value == "true")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_sets_optional_values() {
        let config = StripeConfig::new(SecretString::from("sk_test_key".to_owned()))
            .expect("config should be valid")
            .webhook_secret(Some(SecretString::from("whsec_test".to_owned())))
            .api_base(Url::parse("https://stripe.example").expect("url should parse"))
            .request_timeout(Duration::from_secs(10))
            .expect("timeout should be valid");

        assert_eq!(config.secret_key().expose_secret(), "sk_test_key");
        assert_eq!(
            config
                .webhook_secret_value()
                .expect("secret should exist")
                .expose_secret(),
            "whsec_test"
        );
        assert_eq!(config.api_base_url().as_str(), "https://stripe.example/");
        assert_eq!(config.request_timeout_value(), Duration::from_secs(10));
    }

    #[test]
    fn config_rejects_invalid_secret_key_prefix() {
        assert!(matches!(
            StripeConfig::new(SecretString::from("not-a-stripe-key".to_owned())),
            Err(PaymentError::InvalidConfiguration(_))
        ));
    }

    #[test]
    fn config_rejects_live_key_in_debug_without_override() {
        if live_mode_allowed() {
            return;
        }

        assert!(matches!(
            StripeConfig::new(SecretString::from("sk_live_payrail".to_owned())),
            Err(PaymentError::InvalidConfiguration(_))
        ));
    }

    #[test]
    fn config_rejects_zero_timeout() {
        assert!(matches!(
            StripeConfig::new(SecretString::from("sk_test_key".to_owned()))
                .expect("config should be valid")
                .request_timeout(Duration::ZERO),
            Err(PaymentError::InvalidConfiguration(_))
        ));
    }
}
