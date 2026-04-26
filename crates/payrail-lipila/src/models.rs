use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub(crate) struct LipilaMobileMoneyCollectionRequest {
    #[serde(rename = "referenceId")]
    pub reference_id: String,
    pub amount: serde_json::Number,
    pub narration: String,
    #[serde(rename = "accountNumber")]
    pub account_number: String,
    pub currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LipilaCollectionResponse {
    #[serde(rename = "referenceId")]
    pub reference_id: String,
    pub currency: String,
    pub amount: serde_json::Number,
    #[serde(rename = "accountNumber")]
    pub account_number: String,
    pub status: String,
    #[serde(rename = "paymentType")]
    pub payment_type: Option<String>,
    #[serde(rename = "externalId")]
    pub external_id: Option<String>,
    pub identifier: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LipilaCallbackPayload {
    #[serde(rename = "referenceId")]
    pub reference_id: String,
    pub currency: String,
    pub amount: serde_json::Number,
    #[serde(rename = "accountNumber")]
    pub account_number: String,
    pub status: String,
    #[serde(rename = "paymentType")]
    pub payment_type: String,
    #[serde(rename = "type")]
    pub transaction_type: String,
    #[serde(rename = "ipAddress")]
    pub ip_address: Option<String>,
    pub identifier: Option<String>,
    pub message: Option<String>,
    #[serde(rename = "externalId")]
    pub external_id: Option<String>,
}
