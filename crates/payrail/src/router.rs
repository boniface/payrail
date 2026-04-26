use std::{collections::HashMap, sync::Arc};

use payrail_core::{
    CapturablePaymentConnector, CaptureRequest, CaptureResponse, CountryCode, CreatePaymentRequest,
    CryptoAsset, CryptoNetwork, PaymentConnector, PaymentError, PaymentEvent, PaymentMethod,
    PaymentProvider, PaymentSession, PaymentStatusResponse, ProviderReference, RefundRequest,
    RefundResponse, StablecoinAsset, WebhookRequest,
};

#[derive(Clone)]
struct ConnectorEntry {
    connector: Arc<dyn PaymentConnector>,
    capturable: Option<Arc<dyn CapturablePaymentConnector>>,
}

/// Provider router used by the facade.
#[derive(Clone)]
pub struct PaymentRouter {
    connectors: HashMap<PaymentProvider, ConnectorEntry>,
    mobile_money_routes: HashMap<CountryCode, PaymentProvider>,
    crypto_routes: HashMap<CryptoRouteKey, PaymentProvider>,
    default_crypto_provider: Option<PaymentProvider>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CryptoRouteKey {
    asset: Option<CryptoAsset>,
    network: Option<CryptoNetwork>,
}

impl Default for PaymentRouter {
    fn default() -> Self {
        let mut mobile_money_routes = HashMap::new();
        mobile_money_routes.insert(
            CountryCode::new("ZM").expect("default country route should be valid"),
            PaymentProvider::Lipila,
        );
        Self {
            connectors: HashMap::new(),
            mobile_money_routes,
            crypto_routes: HashMap::new(),
            default_crypto_provider: None,
        }
    }
}

impl std::fmt::Debug for PaymentRouter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PaymentRouter")
            .field("providers", &self.connectors.keys().collect::<Vec<_>>())
            .field("mobile_money_routes", &self.mobile_money_routes)
            .field("crypto_routes", &self.crypto_routes)
            .field("default_crypto_provider", &self.default_crypto_provider)
            .finish()
    }
}

impl PaymentRouter {
    /// Creates an empty router.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a connector.
    pub fn register(&mut self, connector: Arc<dyn PaymentConnector>) {
        self.connectors.insert(
            connector.provider(),
            ConnectorEntry {
                connector,
                capturable: None,
            },
        );
    }

    /// Registers a connector with capture capability.
    pub fn register_capturable(
        &mut self,
        connector: Arc<dyn PaymentConnector>,
        capturable: Arc<dyn CapturablePaymentConnector>,
    ) {
        self.connectors.insert(
            connector.provider(),
            ConnectorEntry {
                connector,
                capturable: Some(capturable),
            },
        );
    }

    /// Routes Mobile Money payments for a country to a configured provider.
    pub fn route_mobile_money(&mut self, country: CountryCode, provider: PaymentProvider) {
        self.mobile_money_routes.insert(country, provider);
    }

    /// Routes crypto and stablecoin payments to a default provider.
    pub fn route_crypto(&mut self, provider: PaymentProvider) {
        self.default_crypto_provider = Some(provider);
    }

    /// Routes crypto and stablecoin payments for a specific asset to a provider.
    pub fn route_crypto_asset(&mut self, asset: CryptoAsset, provider: PaymentProvider) {
        self.crypto_routes.insert(
            CryptoRouteKey {
                asset: Some(asset),
                network: None,
            },
            provider,
        );
    }

    /// Routes crypto payments on a specific network to a provider.
    pub fn route_crypto_network(&mut self, network: CryptoNetwork, provider: PaymentProvider) {
        self.crypto_routes.insert(
            CryptoRouteKey {
                asset: None,
                network: Some(network),
            },
            provider,
        );
    }

    /// Routes crypto payments for a specific asset and network to a provider.
    pub fn route_crypto_asset_network(
        &mut self,
        asset: CryptoAsset,
        network: CryptoNetwork,
        provider: PaymentProvider,
    ) {
        self.crypto_routes.insert(
            CryptoRouteKey {
                asset: Some(asset),
                network: Some(network),
            },
            provider,
        );
    }

    /// Creates a payment by routing from payment method.
    ///
    /// # Errors
    ///
    /// Returns an error when no route or connector is available.
    pub async fn create_payment(
        &self,
        request: CreatePaymentRequest,
    ) -> Result<PaymentSession, PaymentError> {
        let provider = self.route_provider(request.payment_method())?;
        self.connector(&provider)?.create_payment(request).await
    }

    /// Gets payment status.
    ///
    /// # Errors
    ///
    /// Returns an error when the provider is not configured.
    pub async fn get_payment_status(
        &self,
        provider: PaymentProvider,
        provider_reference: &ProviderReference,
    ) -> Result<PaymentStatusResponse, PaymentError> {
        self.connector(&provider)?
            .get_payment_status(provider_reference)
            .await
    }

    /// Refunds a payment.
    ///
    /// # Errors
    ///
    /// Returns an error when the provider is not configured.
    pub async fn refund_payment(
        &self,
        request: RefundRequest,
    ) -> Result<RefundResponse, PaymentError> {
        self.connector(&request.provider)?
            .refund_payment(request)
            .await
    }

    /// Captures a payment through a capture-capable connector.
    ///
    /// # Errors
    ///
    /// Returns an error when the provider is not configured or does not support capture.
    pub async fn capture_payment(
        &self,
        request: CaptureRequest,
    ) -> Result<CaptureResponse, PaymentError> {
        let entry = self.entry(&request.provider)?;
        let connector = entry.capturable.as_ref().ok_or_else(|| {
            PaymentError::UnsupportedOperation(format!(
                "{:?} capture is not supported",
                request.provider
            ))
        })?;

        connector.capture_payment(request).await
    }

    /// Parses a provider webhook.
    ///
    /// # Errors
    ///
    /// Returns an error when the provider is not configured or parsing fails.
    pub async fn parse_webhook(
        &self,
        provider: PaymentProvider,
        request: WebhookRequest<'_>,
    ) -> Result<PaymentEvent, PaymentError> {
        self.connector(&provider)?.parse_webhook(request).await
    }

    fn connector(
        &self,
        provider: &PaymentProvider,
    ) -> Result<&Arc<dyn PaymentConnector>, PaymentError> {
        Ok(&self.entry(provider)?.connector)
    }

    fn entry(&self, provider: &PaymentProvider) -> Result<&ConnectorEntry, PaymentError> {
        self.connectors
            .get(provider)
            .ok_or_else(|| PaymentError::ConnectorNotConfigured {
                provider: provider.clone(),
            })
    }

    fn route_provider(&self, method: &PaymentMethod) -> Result<PaymentProvider, PaymentError> {
        match method {
            PaymentMethod::Card(_) => Ok(PaymentProvider::Stripe),
            PaymentMethod::Stablecoin(method) => {
                let asset = method.preferred_asset.as_ref().map(CryptoAsset::from);
                if let Some(provider) = self.route_crypto_provider(asset.as_ref(), None) {
                    return Ok(provider);
                }

                if Self::uses_stripe_default_stablecoin(method.preferred_asset.as_ref()) {
                    return Ok(PaymentProvider::Stripe);
                }

                Err(PaymentError::UnsupportedPaymentRoute {
                    method: "stablecoin".to_owned(),
                    country: None,
                })
            }
            PaymentMethod::Crypto(method) => self
                .route_crypto_provider(Some(&method.asset), method.network.as_ref())
                .ok_or_else(|| PaymentError::UnsupportedPaymentRoute {
                    method: "crypto".to_owned(),
                    country: None,
                }),
            PaymentMethod::PayPal(_) => Ok(PaymentProvider::PayPal),
            PaymentMethod::MobileMoney(method) => self
                .mobile_money_routes
                .get(&method.country)
                .cloned()
                .ok_or_else(|| PaymentError::UnsupportedPaymentRoute {
                    method: "mobile_money".to_owned(),
                    country: Some(method.country.clone()),
                }),
            _ => Err(PaymentError::UnsupportedPaymentRoute {
                method: "unknown".to_owned(),
                country: None,
            }),
        }
    }

    fn route_crypto_provider(
        &self,
        asset: Option<&CryptoAsset>,
        network: Option<&CryptoNetwork>,
    ) -> Option<PaymentProvider> {
        asset
            .zip(network)
            .and_then(|(asset, network)| {
                self.crypto_routes
                    .get(&CryptoRouteKey {
                        asset: Some(asset.clone()),
                        network: Some(network.clone()),
                    })
                    .cloned()
            })
            .or_else(|| {
                asset.and_then(|asset| {
                    self.crypto_routes
                        .get(&CryptoRouteKey {
                            asset: Some(asset.clone()),
                            network: None,
                        })
                        .cloned()
                })
            })
            .or_else(|| {
                network.and_then(|network| {
                    self.crypto_routes
                        .get(&CryptoRouteKey {
                            asset: None,
                            network: Some(network.clone()),
                        })
                        .cloned()
                })
            })
            .or_else(|| self.default_crypto_provider.clone())
    }

    fn uses_stripe_default_stablecoin(asset: Option<&StablecoinAsset>) -> bool {
        matches!(asset, None | Some(StablecoinAsset::Usdc))
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use http::HeaderMap;
    use payrail_core::{
        CaptureResponse, CryptoAsset, CryptoNetwork, IdempotencyKey, Money, PaymentStatus,
    };

    use super::*;

    #[derive(Debug)]
    struct MockConnector {
        provider: PaymentProvider,
    }

    #[async_trait]
    impl PaymentConnector for MockConnector {
        fn provider(&self) -> PaymentProvider {
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
            Ok(PaymentEvent {
                id: None,
                provider: self.provider.clone(),
                provider_reference: ProviderReference::new("provider-ref")?,
                merchant_reference: None,
                status: PaymentStatus::Succeeded,
                amount: None,
                event_type: payrail_core::PaymentEventType::PaymentSucceeded,
                message: None,
            })
        }
    }

    #[async_trait]
    impl CapturablePaymentConnector for MockConnector {
        async fn capture_payment(
            &self,
            request: CaptureRequest,
        ) -> Result<CaptureResponse, PaymentError> {
            Ok(CaptureResponse {
                provider: self.provider.clone(),
                provider_reference: request.provider_reference,
                status: PaymentStatus::Succeeded,
            })
        }
    }

    #[tokio::test]
    async fn routes_card_to_stripe() {
        let mut router = PaymentRouter::new();
        router.register(Arc::new(MockConnector {
            provider: PaymentProvider::Stripe,
        }));
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(100, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::card())
            .build()
            .expect("request should be valid");

        let session = router
            .create_payment(request)
            .await
            .expect("session should be created");

        assert_eq!(session.provider, PaymentProvider::Stripe);
    }

    #[tokio::test]
    async fn mobile_money_route_can_be_overridden_for_new_aggregators() {
        let provider = PaymentProvider::Other("flutterwave".to_owned());
        let mut router = PaymentRouter::new();
        router.route_mobile_money(
            CountryCode::new("ZM").expect("country should be valid"),
            provider.clone(),
        );
        router.register(Arc::new(MockConnector {
            provider: provider.clone(),
        }));
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(100, "ZMW").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(
                PaymentMethod::mobile_money_zambia("260971234567").expect("method should be valid"),
            )
            .build()
            .expect("request should be valid");

        let session = router
            .create_payment(request)
            .await
            .expect("session should be created");

        assert_eq!(session.provider, provider);
    }

    #[tokio::test]
    async fn crypto_route_can_target_wallet_or_crypto_providers() {
        let provider = PaymentProvider::Other("coinbase".to_owned());
        let mut router = PaymentRouter::new();
        router.route_crypto(provider.clone());
        router.register(Arc::new(MockConnector {
            provider: provider.clone(),
        }));
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(100, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::crypto(CryptoAsset::Btc))
            .build()
            .expect("request should be valid");

        let session = router
            .create_payment(request)
            .await
            .expect("session should be created");

        assert_eq!(session.provider, provider);
    }

    #[tokio::test]
    async fn crypto_asset_network_route_takes_precedence() {
        let default_provider = PaymentProvider::Other("coinbase".to_owned());
        let base_usdc_provider = PaymentProvider::Other("bridge".to_owned());
        let mut router = PaymentRouter::new();
        router.route_crypto(default_provider.clone());
        router.route_crypto_asset_network(
            CryptoAsset::Usdc,
            CryptoNetwork::Base,
            base_usdc_provider.clone(),
        );
        router.register(Arc::new(MockConnector {
            provider: default_provider,
        }));
        router.register(Arc::new(MockConnector {
            provider: base_usdc_provider.clone(),
        }));
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(100, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::usdc_on(CryptoNetwork::Base))
            .build()
            .expect("request should be valid");

        let session = router
            .create_payment(request)
            .await
            .expect("session should be created");

        assert_eq!(session.provider, base_usdc_provider);
    }

    #[tokio::test]
    async fn stablecoin_usdt_requires_explicit_route() {
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(100, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::stablecoin_usdt())
            .build()
            .expect("request should be valid");

        assert!(matches!(
            PaymentRouter::new().create_payment(request).await,
            Err(PaymentError::UnsupportedPaymentRoute { method, .. }) if method == "stablecoin"
        ));
    }

    #[tokio::test]
    async fn stablecoin_usdt_asset_route_targets_configured_provider() {
        let provider = PaymentProvider::Other("binance".to_owned());
        let mut router = PaymentRouter::new();
        router.route_crypto_asset(CryptoAsset::Usdt, provider.clone());
        router.register(Arc::new(MockConnector {
            provider: provider.clone(),
        }));
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(100, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::stablecoin(StablecoinAsset::Usdt))
            .build()
            .expect("request should be valid");

        let session = router
            .create_payment(request)
            .await
            .expect("session should be created");

        assert_eq!(session.provider, provider);
    }

    #[tokio::test]
    async fn generic_crypto_requires_explicit_route() {
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(100, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::crypto(CryptoAsset::Btc))
            .build()
            .expect("request should be valid");

        assert!(matches!(
            PaymentRouter::new().create_payment(request).await,
            Err(PaymentError::UnsupportedPaymentRoute { method, .. }) if method == "crypto"
        ));
    }

    #[tokio::test]
    async fn rejects_unconfigured_connector() {
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(100, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::paypal())
            .build()
            .expect("request should be valid");

        assert!(matches!(
            PaymentRouter::new().create_payment(request).await,
            Err(PaymentError::ConnectorNotConfigured {
                provider: PaymentProvider::PayPal
            })
        ));
    }

    #[tokio::test]
    async fn routes_status_refund_capture_and_webhook() {
        let connector = Arc::new(MockConnector {
            provider: PaymentProvider::PayPal,
        });
        let mut router = PaymentRouter::new();
        let payment_connector: Arc<dyn PaymentConnector> = connector.clone();
        let capturable: Arc<dyn CapturablePaymentConnector> = connector;
        router.register_capturable(payment_connector, capturable);
        let reference = ProviderReference::new("provider-ref").expect("reference should be valid");

        let status = router
            .get_payment_status(PaymentProvider::PayPal, &reference)
            .await
            .expect("status should route");
        let capture = router
            .capture_payment(CaptureRequest {
                provider: PaymentProvider::PayPal,
                provider_reference: reference.clone(),
                idempotency_key: IdempotencyKey::new("ORDER-1:capture")
                    .expect("key should be valid"),
            })
            .await
            .expect("capture should route");
        let refund = router
            .refund_payment(RefundRequest {
                provider: PaymentProvider::PayPal,
                provider_reference: reference.clone(),
                idempotency_key: IdempotencyKey::new("ORDER-1:refund")
                    .expect("key should be valid"),
                amount: None,
                reason: None,
            })
            .await
            .expect("refund should route");
        let event = router
            .parse_webhook(
                PaymentProvider::PayPal,
                WebhookRequest::new(b"{}", HeaderMap::new()),
            )
            .await
            .expect("webhook should route");

        assert_eq!(status.status, PaymentStatus::Created);
        assert_eq!(capture.status, PaymentStatus::Succeeded);
        assert_eq!(refund.status, PaymentStatus::Refunded);
        assert_eq!(event.status, PaymentStatus::Succeeded);
    }

    #[tokio::test]
    async fn rejects_capture_for_non_capturable_connector() {
        let mut router = PaymentRouter::new();
        router.register(Arc::new(MockConnector {
            provider: PaymentProvider::Stripe,
        }));
        let reference = ProviderReference::new("provider-ref").expect("reference should be valid");

        assert!(matches!(
            router
                .capture_payment(CaptureRequest {
                    provider: PaymentProvider::Stripe,
                    provider_reference: reference,
                    idempotency_key: IdempotencyKey::new("ORDER-1:capture")
                        .expect("key should be valid"),
                })
                .await,
            Err(PaymentError::UnsupportedOperation(_))
        ));
    }
}
