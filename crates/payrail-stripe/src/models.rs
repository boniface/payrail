use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct StripeCheckoutSession {
    pub id: String,
    pub payment_intent: Option<String>,
    pub url: Option<String>,
    pub payment_status: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StripeRefund {
    pub id: String,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StripeEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: StripeEventData,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StripeEventData {
    pub object: serde_json::Value,
}
