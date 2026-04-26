use crate::{
    MerchantReference, PaymentError, PaymentEvent, PaymentProvider, ProviderReference,
    WebhookEventId,
};

use super::{mapper::map_event, models::PayPalWebhookEvent};

pub(crate) fn parse_event(payload: &[u8]) -> Result<PaymentEvent, PaymentError> {
    let event: PayPalWebhookEvent = serde_json::from_slice(payload)?;
    let (event_type, status) = map_event(&event.event_type);
    let provider_reference = event
        .resource
        .get("id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            PaymentError::InvalidWebhookPayload("missing paypal resource id".to_owned())
        })?;
    let merchant_reference = event
        .resource
        .get("purchase_units")
        .and_then(serde_json::Value::as_array)
        .and_then(|items| items.first())
        .and_then(|item| item.get("reference_id"))
        .and_then(serde_json::Value::as_str)
        .map(MerchantReference::new)
        .transpose()?;

    Ok(PaymentEvent {
        id: event.id.map(WebhookEventId::new).transpose()?,
        provider: PaymentProvider::PayPal,
        provider_reference: ProviderReference::new(provider_reference)?,
        merchant_reference,
        status,
        amount: None,
        event_type,
        message: None,
    })
}

#[cfg(test)]
mod tests {
    use crate::{PaymentEventType, PaymentStatus};

    use super::*;

    #[test]
    fn parse_event_normalizes_order_completed() {
        let payload = br#"{
            "id":"WH-1",
            "event_type":"CHECKOUT.ORDER.COMPLETED",
            "resource":{
                "id":"ORDER-1",
                "purchase_units":[{"reference_id":"MERCHANT-1"}]
            }
        }"#;

        let event = parse_event(payload).expect("event should parse");

        assert_eq!(event.provider_reference.as_str(), "ORDER-1");
        assert_eq!(
            event
                .merchant_reference
                .expect("merchant reference should exist")
                .as_str(),
            "MERCHANT-1"
        );
        assert_eq!(event.status, PaymentStatus::Succeeded);
        assert_eq!(event.event_type, PaymentEventType::PaymentSucceeded);
    }
}
