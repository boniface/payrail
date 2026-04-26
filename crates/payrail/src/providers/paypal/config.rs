use std::time::Duration;

use crate::PaymentError;
use secrecy::SecretString;
use url::Url;

const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// PayPal environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayPalEnvironment {
    /// Sandbox.
    Sandbox,
    /// Production.
    Production,
}

/// PayPal connector configuration.
#[derive(Debug, Clone)]
pub struct PayPalConfig {
    client_id: SecretString,
    client_secret: SecretString,
    webhook_id: Option<String>,
    environment: PayPalEnvironment,
    base_url: Url,
    request_timeout: Duration,
}

impl PayPalConfig {
    /// Creates sandbox config.
    ///
    /// # Errors
    ///
    /// Returns an error when the default URL cannot be parsed.
    pub fn sandbox(
        client_id: SecretString,
        client_secret: SecretString,
    ) -> Result<Self, PaymentError> {
        Self::new(client_id, client_secret, PayPalEnvironment::Sandbox)
    }

    /// Creates production config.
    ///
    /// # Errors
    ///
    /// Returns an error when the default URL cannot be parsed.
    pub fn production(
        client_id: SecretString,
        client_secret: SecretString,
    ) -> Result<Self, PaymentError> {
        reject_live_mode("paypal production mode")?;
        Self::new(client_id, client_secret, PayPalEnvironment::Production)
    }

    fn new(
        client_id: SecretString,
        client_secret: SecretString,
        environment: PayPalEnvironment,
    ) -> Result<Self, PaymentError> {
        let base_url = match environment {
            PayPalEnvironment::Sandbox => "https://api-m.sandbox.paypal.com",
            PayPalEnvironment::Production => "https://api-m.paypal.com",
        };
        Ok(Self {
            client_id,
            client_secret,
            webhook_id: None,
            environment,
            base_url: Url::parse(base_url)
                .map_err(|error| PaymentError::InvalidConfiguration(error.to_string()))?,
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
        })
    }

    /// Sets the PayPal webhook ID required for webhook signature verification.
    ///
    /// # Errors
    ///
    /// Returns an error when the webhook ID is blank.
    pub fn webhook_id(mut self, webhook_id: impl Into<String>) -> Result<Self, PaymentError> {
        let webhook_id = webhook_id.into();
        if webhook_id.trim().is_empty() {
            return Err(PaymentError::InvalidConfiguration(
                "paypal webhook id cannot be empty".to_owned(),
            ));
        }

        self.webhook_id = Some(webhook_id);
        Ok(self)
    }

    /// Overrides the base URL.
    #[must_use]
    pub fn base_url(mut self, base_url: Url) -> Self {
        self.base_url = base_url;
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
                "paypal request timeout cannot be zero".to_owned(),
            ));
        }

        self.request_timeout = timeout;
        Ok(self)
    }

    pub(crate) const fn client_id(&self) -> &SecretString {
        &self.client_id
    }

    pub(crate) const fn client_secret(&self) -> &SecretString {
        &self.client_secret
    }

    pub(crate) fn webhook_id_value(&self) -> Option<&str> {
        self.webhook_id.as_deref()
    }

    pub(crate) const fn base_url_value(&self) -> &Url {
        &self.base_url
    }

    pub(crate) const fn request_timeout_value(&self) -> Duration {
        self.request_timeout
    }

    /// Returns the configured environment.
    #[inline]
    #[must_use]
    pub const fn environment(&self) -> PayPalEnvironment {
        self.environment
    }
}

fn reject_live_mode(label: &str) -> Result<(), PaymentError> {
    if cfg!(debug_assertions) && !live_mode_allowed() {
        return Err(PaymentError::InvalidConfiguration(format!(
            "{label} requires PAYRAIL_ALLOW_LIVE_TESTS=true in debug or test builds"
        )));
    }

    Ok(())
}

fn live_mode_allowed() -> bool {
    std::env::var("PAYRAIL_ALLOW_LIVE_TESTS").is_ok_and(|value| value == "true")
}

#[cfg(test)]
mod tests {
    use secrecy::ExposeSecret;

    use super::*;

    #[test]
    fn sandbox_sets_expected_values() {
        let sandbox = PayPalConfig::sandbox(
            SecretString::from("client".to_owned()),
            SecretString::from("secret".to_owned()),
        )
        .expect("sandbox config should be valid")
        .webhook_id("WH-123")
        .expect("webhook id should be valid")
        .request_timeout(Duration::from_secs(10))
        .expect("timeout should be valid");

        assert_eq!(sandbox.environment(), PayPalEnvironment::Sandbox);
        assert_eq!(sandbox.client_id().expose_secret(), "client");
        assert_eq!(sandbox.client_secret().expose_secret(), "secret");
        assert_eq!(sandbox.webhook_id_value(), Some("WH-123"));
        assert_eq!(
            sandbox.base_url_value().as_str(),
            "https://api-m.sandbox.paypal.com/"
        );
        assert_eq!(sandbox.request_timeout_value(), Duration::from_secs(10));
    }

    #[test]
    fn production_requires_live_override_in_debug() {
        if live_mode_allowed() {
            return;
        }

        assert!(matches!(
            PayPalConfig::production(
                SecretString::from("client".to_owned()),
                SecretString::from("secret".to_owned())
            ),
            Err(PaymentError::InvalidConfiguration(_))
        ));
    }

    #[test]
    fn rejects_zero_timeout() {
        assert!(matches!(
            PayPalConfig::sandbox(
                SecretString::from("client".to_owned()),
                SecretString::from("secret".to_owned())
            )
            .expect("config should be valid")
            .request_timeout(Duration::ZERO),
            Err(PaymentError::InvalidConfiguration(_))
        ));
    }
}
