use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub(super) struct LipilaMobileMoneyCollectionRequest {
    #[serde(rename = "referenceId")]
    pub(super) reference_id: String,
    pub(super) amount: serde_json::Number,
    pub(super) narration: String,
    #[serde(rename = "accountNumber")]
    pub(super) account_number: String,
    pub(super) currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct LipilaCollectionResponse {
    #[serde(rename = "referenceId")]
    pub(super) reference_id: String,
    pub(super) currency: String,
    pub(super) amount: serde_json::Number,
    #[serde(rename = "accountNumber")]
    pub(super) account_number: String,
    pub(super) status: String,
    #[serde(rename = "paymentType")]
    pub(super) payment_type: Option<String>,
    #[serde(rename = "externalId")]
    pub(super) external_id: Option<String>,
    pub(super) identifier: Option<String>,
    pub(super) message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct LipilaCallbackPayload {
    #[serde(rename = "referenceId")]
    pub(super) reference_id: String,
    pub(super) currency: String,
    pub(super) amount: serde_json::Number,
    #[serde(rename = "accountNumber")]
    pub(super) account_number: String,
    pub(super) status: String,
    #[serde(rename = "paymentType")]
    pub(super) payment_type: String,
    #[serde(rename = "type")]
    pub(super) transaction_type: String,
    #[serde(rename = "ipAddress")]
    pub(super) ip_address: Option<String>,
    pub(super) identifier: Option<String>,
    pub(super) message: Option<String>,
    #[serde(rename = "externalId")]
    pub(super) external_id: Option<String>,
}
