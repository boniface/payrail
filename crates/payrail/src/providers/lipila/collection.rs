use crate::{CreatePaymentRequest, PaymentError};
use serde_json::Number;

use super::models::LipilaMobileMoneyCollectionRequest;

pub(crate) fn collection_request(
    request: &CreatePaymentRequest,
) -> Result<LipilaMobileMoneyCollectionRequest, PaymentError> {
    let phone = match request.payment_method() {
        crate::PaymentMethod::MobileMoney(method) => &method.phone_number,
        crate::PaymentMethod::Card(_)
        | crate::PaymentMethod::Crypto(_)
        | crate::PaymentMethod::Stablecoin(_)
        | crate::PaymentMethod::PayPal(_) => {
            return Err(PaymentError::UnsupportedPaymentMethod(
                "lipila only supports mobile money routes".to_owned(),
            ));
        }
    };
    let major_amount = request
        .amount()
        .currency()
        .minor_units_to_major_integer(request.amount().amount().value())?;

    Ok(LipilaMobileMoneyCollectionRequest {
        reference_id: request.reference().as_str().to_owned(),
        amount: Number::from(major_amount),
        narration: request
            .description()
            .unwrap_or("PayRail payment")
            .to_owned(),
        account_number: phone.digits().to_owned(),
        currency: request.amount().currency().as_str().to_owned(),
        email: request
            .customer()
            .and_then(|customer| customer.email().map(str::to_owned)),
    })
}

#[cfg(test)]
mod tests {
    use crate::{Money, PaymentMethod};

    use super::*;

    #[test]
    fn collection_request_uses_mobile_money_phone() {
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(5_000, "ZMW").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(
                PaymentMethod::mobile_money_zambia("260971234567").expect("method should be valid"),
            )
            .build()
            .expect("request should be valid");

        let body = collection_request(&request).expect("body should build");

        assert_eq!(body.account_number, "260971234567");
        assert_eq!(body.amount, Number::from(50));
    }

    #[test]
    fn collection_request_rejects_fractional_major_units() {
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(5_050, "ZMW").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(
                PaymentMethod::mobile_money_zambia("260971234567").expect("method should be valid"),
            )
            .build()
            .expect("request should be valid");

        assert!(matches!(
            collection_request(&request),
            Err(PaymentError::InvalidAmount(5_050))
        ));
    }
}
