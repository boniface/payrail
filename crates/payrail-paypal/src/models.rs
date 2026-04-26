use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(crate) struct TokenResponse {
    pub access_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PayPalOrder {
    pub id: String,
    pub status: String,
    #[serde(default)]
    pub links: Vec<PayPalLink>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PayPalLink {
    pub href: String,
    pub rel: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PayPalWebhookEvent {
    pub id: Option<String>,
    pub event_type: String,
    pub resource: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub(crate) struct VerifyWebhookSignatureRequest<'a> {
    pub auth_algo: &'a str,
    pub cert_url: &'a str,
    pub transmission_id: &'a str,
    pub transmission_sig: &'a str,
    pub transmission_time: &'a str,
    pub webhook_id: &'a str,
    pub webhook_event: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub(crate) struct VerifyWebhookSignatureResponse {
    pub verification_status: String,
}
