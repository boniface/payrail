use payrail::{CreatePaymentRequest, Money, PayRail, PaymentMethod, StripeConfig};
use secrecy::SecretString;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PayRail::builder()
        .stripe(StripeConfig::new(SecretString::from(std::env::var(
            "STRIPE_SECRET_KEY",
        )?))?)?
        .build()?;

    let request = CreatePaymentRequest::builder()
        .amount(Money::new_minor(2_500, "USD")?)
        .reference("ORDER-1001")?
        .payment_method(PaymentMethod::card())
        .return_url("https://example.com/success")?
        .cancel_url("https://example.com/cancel")?
        .idempotency_key("ORDER-1001:create")?
        .build()?;

    let session = client.create_payment(request).await?;
    println!("{}", session.provider_reference.as_str());

    Ok(())
}
