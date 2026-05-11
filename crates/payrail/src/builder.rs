use crate::{BuiltinProvider, CountryCode, CryptoAsset, CryptoNetwork, PaymentError};

use crate::{PayRailClient, PaymentRouter};

/// Builder for [`PayRailClient`].
#[derive(Clone, Default)]
#[must_use]
pub struct PayRailBuilder {
    router: PaymentRouter,
}

impl std::fmt::Debug for PayRailBuilder {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PayRailBuilder")
            .field("router", &self.router)
            .finish()
    }
}

impl PayRailBuilder {
    /// Routes Mobile Money payments for a country to a built-in provider.
    ///
    /// The selected provider must also have a connector registered by enabling and configuring its
    /// feature. Otherwise routed payments return `ConnectorNotConfigured`.
    pub fn mobile_money_route(mut self, country: CountryCode, provider: BuiltinProvider) -> Self {
        self.router.route_mobile_money(country, provider);
        self
    }

    /// Routes crypto and stablecoin payments to a default built-in provider.
    ///
    /// The selected provider must also have a connector registered by enabling and configuring its
    /// feature. Otherwise routed payments return `ConnectorNotConfigured`.
    pub fn crypto_route(mut self, provider: BuiltinProvider) -> Self {
        self.router.route_crypto(provider);
        self
    }

    /// Routes crypto and stablecoin payments for a specific asset to a built-in provider.
    pub fn crypto_asset_route(mut self, asset: CryptoAsset, provider: BuiltinProvider) -> Self {
        self.router.route_crypto_asset(asset, provider);
        self
    }

    /// Routes crypto payments on a specific network to a built-in provider.
    pub fn crypto_network_route(
        mut self,
        network: CryptoNetwork,
        provider: BuiltinProvider,
    ) -> Self {
        self.router.route_crypto_network(network, provider);
        self
    }

    /// Routes crypto payments for a specific asset and network to a built-in provider.
    pub fn crypto_asset_network_route(
        mut self,
        asset: CryptoAsset,
        network: CryptoNetwork,
        provider: BuiltinProvider,
    ) -> Self {
        self.router
            .route_crypto_asset_network(asset, network, provider);
        self
    }

    /// Registers Stripe.
    #[cfg(feature = "stripe")]
    pub fn stripe(mut self, config: crate::StripeConfig) -> Result<Self, PaymentError> {
        self.router
            .register_stripe(crate::StripeConnector::new(config)?);
        Ok(self)
    }

    /// Registers PayPal.
    #[cfg(feature = "paypal")]
    pub fn paypal(mut self, config: crate::PayPalConfig) -> Result<Self, PaymentError> {
        self.router
            .register_paypal(crate::PayPalConnector::new(config)?);
        Ok(self)
    }

    /// Registers Lipila.
    #[cfg(feature = "lipila")]
    pub fn lipila(mut self, config: crate::LipilaConfig) -> Result<Self, PaymentError> {
        self.router
            .register_lipila(crate::LipilaConnector::new(config)?);
        Ok(self)
    }

    /// Builds the client.
    ///
    /// # Errors
    ///
    /// Currently never fails; returns `Result` for forward-compatible validation.
    pub fn build(self) -> Result<PayRailClient, PaymentError> {
        Ok(PayRailClient::new(self.router))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        CreatePaymentRequest, CryptoAsset, CryptoNetwork, Money, PaymentMethod, PaymentProvider,
    };

    use super::*;

    #[test]
    fn builder_builds_client() {
        let client = PayRailBuilder::default()
            .build()
            .expect("client should build");

        assert!(format!("{client:?}").contains("PayRailClient"));
    }

    #[tokio::test]
    async fn builder_registers_builtin_mobile_money_route() {
        let client = PayRailBuilder::default()
            .mobile_money_route(
                crate::CountryCode::new("ZM").expect("country should be valid"),
                BuiltinProvider::MtnMomo,
            )
            .build()
            .expect("client should build");
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(100, "ZMW").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(
                PaymentMethod::mobile_money_zambia("260971234567").expect("method should be valid"),
            )
            .build()
            .expect("request should be valid");

        assert!(matches!(
            client.create_payment(request).await,
            Err(PaymentError::ConnectorNotConfigured {
                provider: PaymentProvider::MtnMomo
            })
        ));
    }

    #[tokio::test]
    async fn builder_registers_builtin_crypto_route() {
        let client = PayRailBuilder::default()
            .crypto_asset_network_route(
                CryptoAsset::Usdc,
                CryptoNetwork::Base,
                BuiltinProvider::Bridge,
            )
            .build()
            .expect("client should build");
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(100, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::usdc_on(CryptoNetwork::Base))
            .build()
            .expect("request should be valid");

        assert!(matches!(
            client.create_payment(request).await,
            Err(PaymentError::ConnectorNotConfigured {
                provider: PaymentProvider::Bridge
            })
        ));
    }
}
