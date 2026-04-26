use std::time::Duration;

use payrail_core::PaymentError;
use secrecy::SecretString;
use url::Url;

const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Lipila environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LipilaEnvironment {
    /// Sandbox.
    Sandbox,
    /// Production.
    Production,
}

/// Lipila connector configuration.
#[derive(Debug, Clone)]
pub struct LipilaConfig {
    api_key: SecretString,
    webhook_secret: Option<SecretString>,
    environment: LipilaEnvironment,
    base_url: Url,
    request_timeout: Duration,
}

impl LipilaConfig {
    /// Creates sandbox config.
    ///
    /// # Errors
    ///
    /// Returns an error when the default URL cannot be parsed.
    pub fn sandbox(api_key: SecretString) -> Result<Self, PaymentError> {
        Self::new(api_key, LipilaEnvironment::Sandbox)
    }

    /// Creates production config.
    ///
    /// # Errors
    ///
    /// Returns an error when the default URL cannot be parsed.
    pub fn production(api_key: SecretString) -> Result<Self, PaymentError> {
        reject_live_mode("lipila production mode")?;
        Self::new(api_key, LipilaEnvironment::Production)
    }

    fn new(api_key: SecretString, environment: LipilaEnvironment) -> Result<Self, PaymentError> {
        let base_url = match environment {
            LipilaEnvironment::Sandbox => "https://api.lipila.dev",
            LipilaEnvironment::Production => "https://blz.lipila.io",
        };
        Ok(Self {
            api_key,
            webhook_secret: None,
            environment,
            base_url: Url::parse(base_url)
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
                "lipila request timeout cannot be zero".to_owned(),
            ));
        }

        self.request_timeout = timeout;
        Ok(self)
    }

    pub(crate) const fn api_key(&self) -> &SecretString {
        &self.api_key
    }

    pub(crate) const fn webhook_secret_value(&self) -> Option<&SecretString> {
        self.webhook_secret.as_ref()
    }

    pub(crate) const fn base_url_value(&self) -> &Url {
        &self.base_url
    }

    pub(crate) const fn request_timeout_value(&self) -> Duration {
        self.request_timeout
    }

    /// Returns the environment.
    #[inline]
    #[must_use]
    pub const fn environment(&self) -> LipilaEnvironment {
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
        let sandbox = LipilaConfig::sandbox(SecretString::from("api-key".to_owned()))
            .expect("sandbox config should be valid")
            .webhook_secret(Some(SecretString::from("webhook-secret".to_owned())))
            .request_timeout(Duration::from_secs(10))
            .expect("timeout should be valid");

        assert_eq!(sandbox.environment(), LipilaEnvironment::Sandbox);
        assert_eq!(sandbox.api_key().expose_secret(), "api-key");
        assert_eq!(
            sandbox
                .webhook_secret_value()
                .expect("webhook secret should exist")
                .expose_secret(),
            "webhook-secret"
        );
        assert_eq!(sandbox.base_url_value().as_str(), "https://api.lipila.dev/");
        assert_eq!(sandbox.request_timeout_value(), Duration::from_secs(10));
    }

    #[test]
    fn production_requires_live_override_in_debug() {
        if live_mode_allowed() {
            return;
        }

        assert!(matches!(
            LipilaConfig::production(SecretString::from("api-key".to_owned())),
            Err(PaymentError::InvalidConfiguration(_))
        ));
    }

    #[test]
    fn rejects_zero_timeout() {
        assert!(matches!(
            LipilaConfig::sandbox(SecretString::from("api-key".to_owned()))
                .expect("config should be valid")
                .request_timeout(Duration::ZERO),
            Err(PaymentError::InvalidConfiguration(_))
        ));
    }
}
