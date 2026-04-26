/// Supported payment providers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PaymentProvider {
    /// Stripe.
    Stripe,
    /// PayPal.
    PayPal,
    /// Lipila.
    Lipila,
    /// Circle.
    Circle,
    /// Coinbase.
    Coinbase,
    /// Bridge.
    Bridge,
    /// Binance.
    Binance,
    /// A provider not yet modeled directly.
    Other(String),
}
