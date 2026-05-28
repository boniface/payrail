use payrail::{
    CheckoutUiMode, CreatePaymentRequest, Customer, Money, NextAction, PayRail, PaymentMethod,
    StripeConfig,
};
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
        .reference("ORDER-1003")?
        .customer(Customer::new().with_email("buyer@example.com"))
        .payment_method(PaymentMethod::card())
        .checkout_ui_mode(CheckoutUiMode::Custom)
        .return_url("https://example.com/stripe/return?session_id={CHECKOUT_SESSION_ID}")?
        .idempotency_key("ORDER-1003:create")?
        .metadata("tenant_id", "tenant_123")
        .metadata("package_id", "package_456")
        .build()?;

    let session = client.create_payment(request).await?;
    if let Some(NextAction::EmbeddedCheckout { client_secret }) = session.next_action() {
        let _client_secret_for_stripe_js = client_secret;
    }

    Ok(())
}
