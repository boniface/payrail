use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(super) struct TokenResponse {
    pub(super) access_token: String,
    pub(super) expires_in: i64,
}

#[derive(Debug, Deserialize)]
pub(super) struct PayPalOrder {
    pub(super) id: String,
    pub(super) status: String,
    #[serde(default)]
    pub(super) links: Vec<PayPalLink>,
}

#[derive(Debug, Deserialize)]
pub(super) struct PayPalLink {
    pub(super) href: String,
    pub(super) rel: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct PayPalWebhookEvent {
    pub(super) id: Option<String>,
    pub(super) event_type: String,
    pub(super) resource: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub(super) struct VerifyWebhookSignatureRequest<'a> {
    pub(super) auth_algo: &'a str,
    pub(super) cert_url: &'a str,
    pub(super) transmission_id: &'a str,
    pub(super) transmission_sig: &'a str,
    pub(super) transmission_time: &'a str,
    pub(super) webhook_id: &'a str,
    pub(super) webhook_event: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub(super) struct VerifyWebhookSignatureResponse {
    pub(super) verification_status: String,
}
