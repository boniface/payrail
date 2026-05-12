use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct StripeCheckoutSession {
    pub(super) id: String,
    pub(super) client_secret: Option<String>,
    pub(super) payment_intent: Option<String>,
    pub(super) url: Option<String>,
    pub(super) payment_status: Option<String>,
    pub(super) status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct StripeRefund {
    pub(super) id: String,
    pub(super) status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct StripeEvent {
    pub(super) id: String,
    #[serde(rename = "type")]
    pub(super) event_type: String,
    pub(super) data: StripeEventData,
}

#[derive(Debug, Deserialize)]
pub(super) struct StripeEventData {
    pub(super) object: serde_json::Value,
}
