use std::sync::Arc;

use crate::{
    CapturablePaymentConnector, CountryCode, CryptoAsset, CryptoNetwork, PaymentConnector,
    PaymentError, PaymentProvider,
};

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
    /// Registers a custom connector.
    pub fn connector(mut self, connector: Arc<dyn PaymentConnector>) -> Self {
        self.router.register(connector);
        self
    }

    /// Registers a custom capture-capable connector.
    pub fn capturable_connector<C>(mut self, connector: Arc<C>) -> Self
    where
        C: PaymentConnector + CapturablePaymentConnector + 'static,
    {
        let payment_connector: Arc<dyn PaymentConnector> = connector.clone();
        let capturable: Arc<dyn CapturablePaymentConnector> = connector;
        self.router
            .register_capturable(payment_connector, capturable);
        self
    }

    /// Routes Mobile Money payments for a country to a provider.
    ///
    /// This is intended for custom Mobile Money providers and aggregators such as MTN MoMo,
    /// M-Pesa, Airtel Money, Flutterwave, or another Lipila-like adapter.
    pub fn mobile_money_route(mut self, country: CountryCode, provider: PaymentProvider) -> Self {
        self.router.route_mobile_money(country, provider);
        self
    }

    /// Routes crypto and stablecoin payments to a default provider.
    ///
    /// This is intended for custom crypto providers such as Circle, Coinbase, Bridge, Binance,
    /// or another wallet/settlement adapter.
    pub fn crypto_route(mut self, provider: PaymentProvider) -> Self {
        self.router.route_crypto(provider);
        self
    }

    /// Routes crypto and stablecoin payments for a specific asset to a provider.
    pub fn crypto_asset_route(mut self, asset: CryptoAsset, provider: PaymentProvider) -> Self {
        self.router.route_crypto_asset(asset, provider);
        self
    }

    /// Routes crypto payments on a specific network to a provider.
    pub fn crypto_network_route(
        mut self,
        network: CryptoNetwork,
        provider: PaymentProvider,
    ) -> Self {
        self.router.route_crypto_network(network, provider);
        self
    }

    /// Routes crypto payments for a specific asset and network to a provider.
    pub fn crypto_asset_network_route(
        mut self,
        asset: CryptoAsset,
        network: CryptoNetwork,
        provider: PaymentProvider,
    ) -> Self {
        self.router
            .route_crypto_asset_network(asset, network, provider);
        self
    }

    /// Registers Stripe.
    #[cfg(feature = "stripe")]
    pub fn stripe(mut self, config: crate::StripeConfig) -> Result<Self, PaymentError> {
        let connector = Arc::new(crate::StripeConnector::new(config)?);
        self.router.register(connector);
        Ok(self)
    }

    /// Registers PayPal.
    #[cfg(feature = "paypal")]
    pub fn paypal(mut self, config: crate::PayPalConfig) -> Result<Self, PaymentError> {
        let connector = Arc::new(crate::PayPalConnector::new(config)?);
        let payment_connector: Arc<dyn PaymentConnector> = connector.clone();
        let capturable: Arc<dyn CapturablePaymentConnector> = connector;
        self.router
            .register_capturable(payment_connector, capturable);
        Ok(self)
    }

    /// Registers Lipila.
    #[cfg(feature = "lipila")]
    pub fn lipila(mut self, config: crate::LipilaConfig) -> Result<Self, PaymentError> {
        let connector = Arc::new(crate::LipilaConnector::new(config)?);
        self.router.register(connector);
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
    use std::sync::Arc;

    use crate::{
        CaptureResponse, CreatePaymentRequest, CryptoAsset, CryptoNetwork, Money, PaymentEvent,
        PaymentMethod, PaymentSession, PaymentStatus, PaymentStatusResponse, ProviderReference,
        RefundRequest, RefundResponse, WebhookRequest,
    };
    use async_trait::async_trait;

    use super::*;

    #[derive(Debug)]
    struct BuilderConnector;

    #[derive(Debug)]
    struct RoutedBuilderConnector {
        provider: crate::PaymentProvider,
    }

    #[async_trait]
    impl PaymentConnector for BuilderConnector {
        fn provider(&self) -> crate::PaymentProvider {
            crate::PaymentProvider::PayPal
        }

        async fn create_payment(
            &self,
            request: CreatePaymentRequest,
        ) -> Result<PaymentSession, PaymentError> {
            PaymentSession::new(
                crate::PaymentProvider::PayPal,
                ProviderReference::new("provider-ref")?,
                request.reference().clone(),
                PaymentStatus::Created,
                None,
            )
        }

        async fn get_payment_status(
            &self,
            provider_reference: &ProviderReference,
        ) -> Result<PaymentStatusResponse, PaymentError> {
            Ok(PaymentStatusResponse {
                provider: crate::PaymentProvider::PayPal,
                provider_reference: provider_reference.clone(),
                status: PaymentStatus::Created,
            })
        }

        async fn refund_payment(
            &self,
            request: RefundRequest,
        ) -> Result<RefundResponse, PaymentError> {
            Ok(RefundResponse {
                provider: request.provider,
                provider_reference: request.provider_reference,
                status: PaymentStatus::Refunded,
            })
        }

        async fn parse_webhook(
            &self,
            _request: WebhookRequest<'_>,
        ) -> Result<PaymentEvent, PaymentError> {
            Err(PaymentError::UnsupportedOperation("webhook".to_owned()))
        }
    }

    #[async_trait]
    impl CapturablePaymentConnector for BuilderConnector {
        async fn capture_payment(
            &self,
            request: crate::CaptureRequest,
        ) -> Result<CaptureResponse, PaymentError> {
            Ok(CaptureResponse {
                provider: crate::PaymentProvider::PayPal,
                provider_reference: request.provider_reference,
                status: PaymentStatus::Succeeded,
            })
        }
    }

    #[async_trait]
    impl PaymentConnector for RoutedBuilderConnector {
        fn provider(&self) -> crate::PaymentProvider {
            self.provider.clone()
        }

        async fn create_payment(
            &self,
            request: CreatePaymentRequest,
        ) -> Result<PaymentSession, PaymentError> {
            PaymentSession::new(
                self.provider.clone(),
                ProviderReference::new("provider-ref")?,
                request.reference().clone(),
                PaymentStatus::Created,
                None,
            )
        }

        async fn get_payment_status(
            &self,
            provider_reference: &ProviderReference,
        ) -> Result<PaymentStatusResponse, PaymentError> {
            Ok(PaymentStatusResponse {
                provider: self.provider.clone(),
                provider_reference: provider_reference.clone(),
                status: PaymentStatus::Created,
            })
        }

        async fn refund_payment(
            &self,
            request: RefundRequest,
        ) -> Result<RefundResponse, PaymentError> {
            Ok(RefundResponse {
                provider: request.provider,
                provider_reference: request.provider_reference,
                status: PaymentStatus::Refunded,
            })
        }

        async fn parse_webhook(
            &self,
            _request: WebhookRequest<'_>,
        ) -> Result<PaymentEvent, PaymentError> {
            Err(PaymentError::UnsupportedOperation("webhook".to_owned()))
        }
    }

    #[test]
    fn builder_registers_connectors() {
        let plain: Arc<dyn PaymentConnector> = Arc::new(BuilderConnector);
        let client = PayRailBuilder::default()
            .connector(plain)
            .capturable_connector(Arc::new(BuilderConnector))
            .build()
            .expect("client should build");

        assert!(format!("{client:?}").contains("PayRailClient"));
    }

    #[tokio::test]
    async fn builder_registers_custom_mobile_money_route() {
        let provider = crate::PaymentProvider::Other("mtn-momo".to_owned());
        let client = PayRailBuilder::default()
            .connector(Arc::new(RoutedBuilderConnector {
                provider: provider.clone(),
            }))
            .mobile_money_route(
                crate::CountryCode::new("ZM").expect("country should be valid"),
                provider.clone(),
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

        let session = client
            .create_payment(request)
            .await
            .expect("session should be created");

        assert_eq!(session.provider, provider);
    }

    #[tokio::test]
    async fn builder_registers_custom_crypto_route() {
        let provider = crate::PaymentProvider::Other("circle".to_owned());
        let client = PayRailBuilder::default()
            .connector(Arc::new(RoutedBuilderConnector {
                provider: provider.clone(),
            }))
            .crypto_asset_network_route(CryptoAsset::Usdc, CryptoNetwork::Base, provider.clone())
            .build()
            .expect("client should build");
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(100, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::usdc_on(CryptoNetwork::Base))
            .build()
            .expect("request should be valid");

        let session = client
            .create_payment(request)
            .await
            .expect("session should be created");

        assert_eq!(session.provider, provider);
    }
}
