use crate::{CreatePaymentRequest, PaymentError};
use serde_json::json;
use url::Url;

pub(crate) fn approval_url(order: &super::models::PayPalOrder) -> Result<Url, PaymentError> {
    let href = order
        .links
        .iter()
        .find(|link| link.rel == "approve")
        .map(|link| link.href.as_str())
        .ok_or_else(|| {
            PaymentError::InvalidWebhookPayload("missing paypal approval url".to_owned())
        })?;

    Url::parse(href).map_err(|error| PaymentError::InvalidUrl(error.to_string()))
}

pub(crate) fn create_order_body(request: &CreatePaymentRequest) -> serde_json::Value {
    let return_url = request.return_url().map(Url::as_str);
    let cancel_url = request.cancel_url().map(Url::as_str);
    json!({
        "intent": "CAPTURE",
        "purchase_units": [{
            "reference_id": request.reference().as_str(),
            "amount": {
                "currency_code": request.amount().currency().as_str(),
                "value": request
                    .amount()
                    .currency()
                    .format_minor_units(request.amount().amount().value())
            },
            "description": request.description()
        }],
        "payment_source": {
            "paypal": {
                "experience_context": {
                    "return_url": return_url,
                    "cancel_url": cancel_url
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::{Money, PaymentMethod};

    use super::*;

    #[test]
    fn create_order_body_uses_minor_units() {
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(1234, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::paypal())
            .build()
            .expect("request should be valid");

        let body = create_order_body(&request);

        assert_eq!(body["purchase_units"][0]["amount"]["value"], "12.34");
    }

    #[test]
    fn create_order_body_respects_zero_decimal_currency() {
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(1234, "JPY").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::paypal())
            .build()
            .expect("request should be valid");

        let body = create_order_body(&request);

        assert_eq!(body["purchase_units"][0]["amount"]["value"], "1234");
    }
}
