use crate::{PaymentError, PaymentProvider, ProviderErrorDetails};
use secrecy::ExposeSecret;
use time::{Duration, OffsetDateTime};
use tokio::sync::RwLock;

use super::{config::PayPalConfig, models::TokenResponse};

#[derive(Debug, Clone)]
struct CachedToken {
    access_token: String,
    expires_at: OffsetDateTime,
}

/// OAuth token cache.
#[derive(Debug, Default)]
pub(crate) struct TokenCache {
    token: RwLock<Option<CachedToken>>,
}

impl TokenCache {
    pub(crate) async fn access_token(
        &self,
        client: &reqwest::Client,
        config: &PayPalConfig,
    ) -> Result<String, PaymentError> {
        if let Some(token) = self.current().await {
            return Ok(token);
        }

        let token = fetch_token(client, config).await?;
        let access_token = token.access_token.clone();
        *self.token.write().await = Some(token);
        Ok(access_token)
    }

    async fn current(&self) -> Option<String> {
        self.token.read().await.as_ref().and_then(|token| {
            (token.expires_at > OffsetDateTime::now_utc() + Duration::seconds(30))
                .then(|| token.access_token.clone())
        })
    }
}

async fn fetch_token(
    client: &reqwest::Client,
    config: &PayPalConfig,
) -> Result<CachedToken, PaymentError> {
    let url = config
        .base_url_value()
        .join("/v1/oauth2/token")
        .map_err(|error| PaymentError::InvalidConfiguration(error.to_string()))?;
    let response = client
        .post(url)
        .basic_auth(
            config.client_id().expose_secret(),
            Some(config.client_secret().expose_secret()),
        )
        .form(&[("grant_type", "client_credentials")])
        .send()
        .await?;
    let status = response.status();
    tracing::debug!(
        provider = "paypal",
        operation = "fetch_access_token",
        status = status.as_u16(),
        "received provider response"
    );
    if !status.is_success() {
        return Err(PaymentError::ProviderDetails {
            details: ProviderErrorDetails {
                provider: PaymentProvider::PayPal,
                status: status.as_u16(),
                code: None,
                request_id: None,
                message: "paypal authentication failed".to_owned(),
            },
        });
    }

    let token = response.json::<TokenResponse>().await?;
    Ok(CachedToken {
        access_token: token.access_token,
        expires_at: OffsetDateTime::now_utc() + Duration::seconds(token.expires_in),
    })
}
