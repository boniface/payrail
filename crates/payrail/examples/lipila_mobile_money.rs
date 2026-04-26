use payrail::{CreatePaymentRequest, LipilaConfig, Money, PayRail, PaymentMethod};
use secrecy::SecretString;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PayRail::builder()
        .lipila(
            LipilaConfig::sandbox(SecretString::from(std::env::var("LIPILA_API_KEY")?))?
                .webhook_secret(
                    std::env::var("LIPILA_WEBHOOK_SECRET")
                        .ok()
                        .map(SecretString::from),
                ),
        )?
        .build()?;

    let request = CreatePaymentRequest::builder()
        .amount(Money::new_minor(5_000, "ZMW")?)
        .reference("ORDER-1004")?
        .description("Order 1004")
        .payment_method(PaymentMethod::mobile_money_zambia("260971234567")?)
        .callback_url("https://example.com/webhooks/lipila")?
        .idempotency_key("ORDER-1004:create")?
        .build()?;

    let session = client.create_payment(request).await?;
    println!("{}", session.provider_reference.as_str());

    Ok(())
}
