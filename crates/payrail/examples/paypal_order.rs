use payrail::{CreatePaymentRequest, Money, PayPalConfig, PayRail, PaymentMethod};
use secrecy::SecretString;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PayRail::builder()
        .paypal(PayPalConfig::sandbox(
            SecretString::from(std::env::var("PAYPAL_CLIENT_ID")?),
            SecretString::from(std::env::var("PAYPAL_CLIENT_SECRET")?),
        )?)?
        .build()?;

    let request = CreatePaymentRequest::builder()
        .amount(Money::new_minor(3_500, "USD")?)
        .reference("ORDER-1003")?
        .payment_method(PaymentMethod::paypal())
        .return_url("https://example.com/paypal/return")?
        .cancel_url("https://example.com/paypal/cancel")?
        .idempotency_key("ORDER-1003:create")?
        .build()?;

    let session = client.create_payment(request).await?;
    println!("{}", session.provider_reference.as_str());

    Ok(())
}
