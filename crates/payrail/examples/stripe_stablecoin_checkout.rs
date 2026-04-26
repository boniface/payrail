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
        .amount(Money::new_minor(10_000, "USD")?)
        .reference("ORDER-1002")?
        .payment_method(PaymentMethod::stablecoin_usdc())
        .return_url("https://example.com/success")?
        .cancel_url("https://example.com/cancel")?
        .idempotency_key("ORDER-1002:create")?
        .build()?;

    let session = client.create_payment(request).await?;
    println!("{}", session.provider_reference.as_str());

    Ok(())
}
