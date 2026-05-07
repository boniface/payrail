/// Built-in payment providers with cheap copy semantics.
///
/// Route configuration uses this type. A provider listed here is routable, but it is only
/// executable once a matching first-party connector is implemented and registered by the builder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BuiltinProvider {
    /// Stripe.
    Stripe,
    /// PayPal.
    PayPal,
    /// Lipila.
    Lipila,
    /// Circle. Reserved for a first-party crypto connector.
    Circle,
    /// Coinbase. Reserved for a first-party crypto connector.
    Coinbase,
    /// Bridge. Reserved for a first-party crypto connector.
    Bridge,
    /// Binance. Reserved for a first-party crypto connector.
    Binance,
    /// MTN MoMo. Reserved for a first-party Mobile Money connector.
    MtnMomo,
    /// M-Pesa. Reserved for a first-party Mobile Money connector.
    Mpesa,
    /// Airtel Money. Reserved for a first-party Mobile Money connector.
    AirtelMoney,
    /// Flutterwave. Reserved for a first-party aggregator connector.
    Flutterwave,
    /// Paystack. Reserved for a first-party aggregator connector.
    Paystack,
}

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
    /// MTN MoMo.
    MtnMomo,
    /// M-Pesa.
    Mpesa,
    /// Airtel Money.
    AirtelMoney,
    /// Flutterwave.
    Flutterwave,
    /// Paystack.
    Paystack,
    /// Provider metadata not modeled directly.
    ///
    /// This variant is for normalized provider/event metadata. It is not accepted by router
    /// configuration APIs.
    Other(String),
}

impl PaymentProvider {
    /// Returns the built-in provider value when this is one of PayRail's modeled providers.
    #[inline]
    #[must_use]
    pub const fn as_builtin(&self) -> Option<BuiltinProvider> {
        match self {
            Self::Stripe => Some(BuiltinProvider::Stripe),
            Self::PayPal => Some(BuiltinProvider::PayPal),
            Self::Lipila => Some(BuiltinProvider::Lipila),
            Self::Circle => Some(BuiltinProvider::Circle),
            Self::Coinbase => Some(BuiltinProvider::Coinbase),
            Self::Bridge => Some(BuiltinProvider::Bridge),
            Self::Binance => Some(BuiltinProvider::Binance),
            Self::MtnMomo => Some(BuiltinProvider::MtnMomo),
            Self::Mpesa => Some(BuiltinProvider::Mpesa),
            Self::AirtelMoney => Some(BuiltinProvider::AirtelMoney),
            Self::Flutterwave => Some(BuiltinProvider::Flutterwave),
            Self::Paystack => Some(BuiltinProvider::Paystack),
            Self::Other(_) => None,
        }
    }

    /// Creates provider metadata for a provider not modeled directly.
    ///
    /// This does not make the provider routable. Runtime route configuration only accepts
    /// [`BuiltinProvider`].
    #[inline]
    #[must_use]
    pub fn other(provider: impl Into<String>) -> Self {
        Self::Other(provider.into())
    }
}

impl From<BuiltinProvider> for PaymentProvider {
    #[inline]
    fn from(provider: BuiltinProvider) -> Self {
        match provider {
            BuiltinProvider::Stripe => Self::Stripe,
            BuiltinProvider::PayPal => Self::PayPal,
            BuiltinProvider::Lipila => Self::Lipila,
            BuiltinProvider::Circle => Self::Circle,
            BuiltinProvider::Coinbase => Self::Coinbase,
            BuiltinProvider::Bridge => Self::Bridge,
            BuiltinProvider::Binance => Self::Binance,
            BuiltinProvider::MtnMomo => Self::MtnMomo,
            BuiltinProvider::Mpesa => Self::Mpesa,
            BuiltinProvider::AirtelMoney => Self::AirtelMoney,
            BuiltinProvider::Flutterwave => Self::Flutterwave,
            BuiltinProvider::Paystack => Self::Paystack,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_providers_round_trip_to_payment_provider() {
        let providers = [
            (BuiltinProvider::Stripe, PaymentProvider::Stripe),
            (BuiltinProvider::PayPal, PaymentProvider::PayPal),
            (BuiltinProvider::Lipila, PaymentProvider::Lipila),
            (BuiltinProvider::Circle, PaymentProvider::Circle),
            (BuiltinProvider::Coinbase, PaymentProvider::Coinbase),
            (BuiltinProvider::Bridge, PaymentProvider::Bridge),
            (BuiltinProvider::Binance, PaymentProvider::Binance),
            (BuiltinProvider::MtnMomo, PaymentProvider::MtnMomo),
            (BuiltinProvider::Mpesa, PaymentProvider::Mpesa),
            (BuiltinProvider::AirtelMoney, PaymentProvider::AirtelMoney),
            (BuiltinProvider::Flutterwave, PaymentProvider::Flutterwave),
            (BuiltinProvider::Paystack, PaymentProvider::Paystack),
        ];

        providers
            .into_iter()
            .for_each(|(builtin, payment_provider)| {
                assert_eq!(PaymentProvider::from(builtin), payment_provider);
                assert_eq!(payment_provider.as_builtin(), Some(builtin));
            });
    }

    #[test]
    fn other_provider_is_metadata_not_builtin_route_target() {
        let provider = PaymentProvider::other("mtn-momo");

        assert_eq!(provider, PaymentProvider::Other("mtn-momo".to_owned()));
        assert_eq!(provider.as_builtin(), None);
    }
}
