use payrail_core::{
    CurrencyCode, MerchantReference, Money, PaymentError, PaymentEvent, PaymentProvider,
    ProviderReference,
};

use crate::{
    mapper::{map_event_type, map_payment_type, map_status},
    models::LipilaCallbackPayload,
};

pub(crate) fn parse_callback(payload: &[u8]) -> Result<PaymentEvent, PaymentError> {
    let callback: LipilaCallbackPayload = serde_json::from_slice(payload)?;
    let status = map_status(&callback.status);
    let _operator = map_payment_type(&callback.payment_type);
    let _redacted_account_number = callback.account_number.as_str();
    let _transaction_type = callback.transaction_type.as_str();
    let _ip_address = callback.ip_address.as_deref();
    let reference = callback
        .external_id
        .as_deref()
        .or(callback.identifier.as_deref())
        .unwrap_or(callback.reference_id.as_str());
    let amount = callback
        .amount
        .as_i64()
        .map(|amount| {
            let currency = CurrencyCode::new(callback.currency.as_str())?;
            let minor = currency.major_integer_to_minor_units(amount)?;
            Money::new_minor(minor, currency.as_str())
        })
        .transpose()?;

    Ok(PaymentEvent {
        id: callback
            .identifier
            .as_deref()
            .map(payrail_core::WebhookEventId::new)
            .transpose()?,
        provider: PaymentProvider::Lipila,
        provider_reference: ProviderReference::new(reference)?,
        merchant_reference: Some(MerchantReference::new(callback.reference_id)?),
        status,
        amount,
        event_type: map_event_type(status),
        message: callback.message,
    })
}

#[cfg(test)]
mod tests {
    use payrail_core::{PaymentEventType, PaymentStatus};

    use super::*;

    #[test]
    fn parse_callback_normalizes_event() {
        let payload = br#"{
            "referenceId":"ORDER-1",
            "currency":"ZMW",
            "amount":50,
            "accountNumber":"260971234567",
            "status":"Successful",
            "paymentType":"MTNMoney",
            "type":"collection",
            "ipAddress":"127.0.0.1",
            "identifier":"evt_1",
            "message":"paid",
            "externalId":"LIPILA-1"
        }"#;

        let event = parse_callback(payload).expect("callback should parse");

        assert_eq!(event.provider_reference.as_str(), "LIPILA-1");
        assert_eq!(event.status, PaymentStatus::Succeeded);
        assert_eq!(event.event_type, PaymentEventType::PaymentSucceeded);
        assert_eq!(
            event
                .amount
                .expect("amount should be present")
                .amount()
                .value(),
            5_000
        );
    }
}
