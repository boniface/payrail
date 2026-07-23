use std::collections::HashMap;

use crate::{
    BuiltinProvider, CaptureRequest, CaptureResponse, CountryCode, CreatePaymentRequest,
    CryptoAsset, CryptoNetwork, PaymentError, PaymentEvent, PaymentMethod, PaymentProvider,
    PaymentSession, PaymentStatusResponse, ProviderReference, RefundRequest, RefundResponse,
    StablecoinAsset, WebhookRequest,
};

#[cfg(feature = "fraud")]
use crate::{FraudEvent, FraudPolicy, RiskAssessment, RiskAwarePaymentSession};
#[cfg(feature = "telemetry")]
use crate::{TelemetryOperation, emit_result, payment_method_kind, provider_name};
#[cfg(feature = "telemetry")]
use tracing::Instrument;

/// Provider router used by the facade.
#[derive(Clone)]
pub struct PaymentRouter {
    #[cfg(feature = "stripe")]
    stripe: Option<crate::StripeConnector>,
    #[cfg(feature = "paypal")]
    paypal: Option<crate::PayPalConnector>,
    #[cfg(feature = "lipila")]
    lipila: Option<crate::LipilaConnector>,
    mobile_money_routes: HashMap<CountryCode, BuiltinProvider>,
    crypto_asset_network_routes: HashMap<CryptoAsset, HashMap<CryptoNetwork, BuiltinProvider>>,
    crypto_asset_routes: HashMap<CryptoAsset, BuiltinProvider>,
    crypto_network_routes: HashMap<CryptoNetwork, BuiltinProvider>,
    default_crypto_provider: Option<BuiltinProvider>,
}

impl Default for PaymentRouter {
    fn default() -> Self {
        let mut mobile_money_routes = HashMap::new();
        mobile_money_routes.insert(CountryCode::zambia(), BuiltinProvider::Lipila);
        Self {
            #[cfg(feature = "stripe")]
            stripe: None,
            #[cfg(feature = "paypal")]
            paypal: None,
            #[cfg(feature = "lipila")]
            lipila: None,
            mobile_money_routes,
            crypto_asset_network_routes: HashMap::new(),
            crypto_asset_routes: HashMap::new(),
            crypto_network_routes: HashMap::new(),
            default_crypto_provider: None,
        }
    }
}

impl std::fmt::Debug for PaymentRouter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PaymentRouter")
            .field("stripe_configured", &self.stripe_configured())
            .field("paypal_configured", &self.paypal_configured())
            .field("lipila_configured", &self.lipila_configured())
            .field("mobile_money_routes", &self.mobile_money_routes)
            .field(
                "crypto_asset_network_routes",
                &self.crypto_asset_network_routes,
            )
            .field("crypto_asset_routes", &self.crypto_asset_routes)
            .field("crypto_network_routes", &self.crypto_network_routes)
            .field("default_crypto_provider", &self.default_crypto_provider)
            .finish_non_exhaustive()
    }
}

impl PaymentRouter {
    /// Creates an empty router.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(feature = "stripe")]
    const fn stripe_configured(&self) -> bool {
        self.stripe.is_some()
    }

    #[cfg(not(feature = "stripe"))]
    const fn stripe_configured(&self) -> bool {
        false
    }

    #[cfg(feature = "paypal")]
    const fn paypal_configured(&self) -> bool {
        self.paypal.is_some()
    }

    #[cfg(not(feature = "paypal"))]
    const fn paypal_configured(&self) -> bool {
        false
    }

    #[cfg(feature = "lipila")]
    const fn lipila_configured(&self) -> bool {
        self.lipila.is_some()
    }

    #[cfg(not(feature = "lipila"))]
    const fn lipila_configured(&self) -> bool {
        false
    }

    /// Registers the built-in Stripe connector on the static dispatch path.
    #[cfg(feature = "stripe")]
    pub(crate) fn register_stripe(&mut self, connector: crate::StripeConnector) {
        self.stripe = Some(connector);
    }

    /// Registers the built-in `PayPal` connector on the static dispatch path.
    #[cfg(feature = "paypal")]
    pub(crate) fn register_paypal(&mut self, connector: crate::PayPalConnector) {
        self.paypal = Some(connector);
    }

    /// Registers the built-in Lipila connector on the static dispatch path.
    #[cfg(feature = "lipila")]
    pub(crate) fn register_lipila(&mut self, connector: crate::LipilaConnector) {
        self.lipila = Some(connector);
    }

    /// Routes Mobile Money payments for a country to a built-in provider.
    ///
    /// This updates route selection only. The selected provider must also have a configured
    /// connector before routed payment execution can succeed.
    pub fn route_mobile_money(&mut self, country: CountryCode, provider: BuiltinProvider) {
        self.mobile_money_routes.insert(country, provider);
    }

    /// Routes crypto and stablecoin payments to a default built-in provider.
    ///
    /// This updates route selection only. The selected provider must also have a configured
    /// connector before routed payment execution can succeed.
    pub const fn route_crypto(&mut self, provider: BuiltinProvider) {
        self.default_crypto_provider = Some(provider);
    }

    /// Routes crypto and stablecoin payments for a specific asset to a built-in provider.
    pub fn route_crypto_asset(&mut self, asset: CryptoAsset, provider: BuiltinProvider) {
        self.crypto_asset_routes.insert(asset, provider);
    }

    /// Routes crypto payments on a specific network to a built-in provider.
    pub fn route_crypto_network(&mut self, network: CryptoNetwork, provider: BuiltinProvider) {
        self.crypto_network_routes.insert(network, provider);
    }

    /// Routes crypto payments for a specific asset and network to a built-in provider.
    pub fn route_crypto_asset_network(
        &mut self,
        asset: CryptoAsset,
        network: CryptoNetwork,
        provider: BuiltinProvider,
    ) {
        self.crypto_asset_network_routes
            .entry(asset)
            .or_default()
            .insert(network, provider);
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
        #[cfg(feature = "telemetry")]
        {
            let span = tracing::debug_span!(
                "payrail.router.payment.create",
                "payrail.operation" = TelemetryOperation::PaymentCreate.as_str(),
                "payrail.payment_method" = payment_method_kind(request.payment_method())
            );
            return async {
                let provider = self.resolve_provider(request.payment_method())?;
                let result = self.create_payment_with_provider(provider, request).await;
                emit_result(
                    TelemetryOperation::PaymentCreate,
                    &result,
                    "router payment create completed",
                );
                result
            }
            .instrument(span)
            .await;
        }

        #[cfg(not(feature = "telemetry"))]
        {
            let provider = self.resolve_provider(request.payment_method())?;
            self.create_payment_with_provider(provider, request).await
        }
    }

    /// Assesses payment risk using the default local fraud policy.
    #[cfg(feature = "fraud")]
    #[inline]
    #[must_use]
    pub fn assess_payment_risk(&self, request: &CreatePaymentRequest) -> RiskAssessment {
        FraudPolicy::default().assess_request(request)
    }

    /// Assesses risk and creates a payment only when the policy allows provider execution.
    ///
    /// Rejected payments return a successful risk-aware result with `payment() == None`, so
    /// callers do not need provider-specific error details to handle fraud policy decisions.
    ///
    /// # Errors
    ///
    /// Returns an error when policy allows execution but routing or provider execution fails.
    #[cfg(feature = "fraud")]
    pub async fn create_payment_with_risk(
        &self,
        request: CreatePaymentRequest,
        policy: &FraudPolicy,
    ) -> Result<RiskAwarePaymentSession, PaymentError> {
        let assessment = policy.assess_request(&request);
        if !policy.allows_payment_creation(assessment.decision()) {
            return Ok(RiskAwarePaymentSession::new(None, assessment));
        }

        let payment = self.create_payment(request).await?;
        Ok(RiskAwarePaymentSession::new(Some(payment), assessment))
    }

    /// Resolves the built-in provider that would handle a payment method.
    ///
    /// This performs route selection only. It does not require the selected connector to be
    /// configured and does not perform provider I/O.
    ///
    /// # Errors
    ///
    /// Returns an error when no route exists for the supplied payment method.
    #[inline]
    pub fn resolve_provider(
        &self,
        method: &PaymentMethod,
    ) -> Result<BuiltinProvider, PaymentError> {
        self.route_provider(method)
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
        #[cfg(feature = "telemetry")]
        {
            let span = tracing::debug_span!(
                "payrail.router.payment.status",
                "payrail.operation" = TelemetryOperation::PaymentStatus.as_str(),
                "payrail.provider" = provider_name(&provider)
            );
            return async {
                let result = self
                    .get_payment_status_inner(provider, provider_reference)
                    .await;
                emit_result(
                    TelemetryOperation::PaymentStatus,
                    &result,
                    "router payment status completed",
                );
                result
            }
            .instrument(span)
            .await;
        }

        #[cfg(not(feature = "telemetry"))]
        self.get_payment_status_inner(provider, provider_reference)
            .await
    }

    async fn get_payment_status_inner(
        &self,
        provider: PaymentProvider,
        provider_reference: &ProviderReference,
    ) -> Result<PaymentStatusResponse, PaymentError> {
        let _ = provider_reference;
        match provider.as_builtin() {
            Some(BuiltinProvider::Stripe) => {
                #[cfg(feature = "stripe")]
                if let Some(connector) = self.stripe.as_ref() {
                    return connector.get_payment_status(provider_reference).await;
                }
                Err(Self::not_configured(BuiltinProvider::Stripe))
            }
            Some(BuiltinProvider::PayPal) => {
                #[cfg(feature = "paypal")]
                if let Some(connector) = self.paypal.as_ref() {
                    return connector.get_payment_status(provider_reference).await;
                }
                Err(Self::not_configured(BuiltinProvider::PayPal))
            }
            Some(BuiltinProvider::Lipila) => {
                #[cfg(feature = "lipila")]
                if let Some(connector) = self.lipila.as_ref() {
                    return connector.get_payment_status(provider_reference).await;
                }
                Err(Self::not_configured(BuiltinProvider::Lipila))
            }
            Some(provider) => Err(Self::not_configured(provider)),
            None => Err(PaymentError::ConnectorNotConfigured { provider }),
        }
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
        #[cfg(feature = "telemetry")]
        {
            let span = tracing::debug_span!(
                "payrail.router.payment.refund",
                "payrail.operation" = TelemetryOperation::PaymentRefund.as_str(),
                "payrail.provider" = provider_name(&request.provider)
            );
            return async {
                let result = self.refund_payment_inner(request).await;
                emit_result(
                    TelemetryOperation::PaymentRefund,
                    &result,
                    "router payment refund completed",
                );
                result
            }
            .instrument(span)
            .await;
        }

        #[cfg(not(feature = "telemetry"))]
        self.refund_payment_inner(request).await
    }

    async fn refund_payment_inner(
        &self,
        request: RefundRequest,
    ) -> Result<RefundResponse, PaymentError> {
        match request.provider.as_builtin() {
            Some(BuiltinProvider::Stripe) => {
                #[cfg(feature = "stripe")]
                if let Some(connector) = self.stripe.as_ref() {
                    return connector.refund_payment(request).await;
                }
                Err(Self::not_configured(BuiltinProvider::Stripe))
            }
            Some(BuiltinProvider::PayPal) => {
                #[cfg(feature = "paypal")]
                if let Some(connector) = self.paypal.as_ref() {
                    return connector.refund_payment(request).await;
                }
                Err(Self::not_configured(BuiltinProvider::PayPal))
            }
            Some(BuiltinProvider::Lipila) => {
                #[cfg(feature = "lipila")]
                if let Some(connector) = self.lipila.as_ref() {
                    return connector.refund_payment(request).await;
                }
                Err(Self::not_configured(BuiltinProvider::Lipila))
            }
            Some(provider) => Err(Self::not_configured(provider)),
            None => Err(PaymentError::ConnectorNotConfigured {
                provider: request.provider,
            }),
        }
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
        #[cfg(feature = "telemetry")]
        {
            let span = tracing::debug_span!(
                "payrail.router.payment.capture",
                "payrail.operation" = TelemetryOperation::PaymentCapture.as_str(),
                "payrail.provider" = provider_name(&request.provider)
            );
            return async {
                let result = self.capture_payment_inner(request).await;
                emit_result(
                    TelemetryOperation::PaymentCapture,
                    &result,
                    "router payment capture completed",
                );
                result
            }
            .instrument(span)
            .await;
        }

        #[cfg(not(feature = "telemetry"))]
        self.capture_payment_inner(request).await
    }

    async fn capture_payment_inner(
        &self,
        request: CaptureRequest,
    ) -> Result<CaptureResponse, PaymentError> {
        match request.provider.as_builtin() {
            Some(BuiltinProvider::PayPal) => {
                #[cfg(feature = "paypal")]
                if let Some(connector) = self.paypal.as_ref() {
                    return connector.capture_payment(request).await;
                }
                Err(Self::not_configured(BuiltinProvider::PayPal))
            }
            Some(provider) => Err(PaymentError::UnsupportedOperation(format!(
                "{:?} capture is not supported",
                PaymentProvider::from(provider)
            ))),
            None => Err(PaymentError::ConnectorNotConfigured {
                provider: request.provider,
            }),
        }
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
        #[cfg(feature = "telemetry")]
        {
            let span = tracing::debug_span!(
                "payrail.router.webhook.parse",
                "payrail.operation" = TelemetryOperation::WebhookParse.as_str(),
                "payrail.provider" = provider_name(&provider),
                "payrail.payload_len" = request.payload.len()
            );
            return async {
                let result = self.parse_webhook_inner(provider, request).await;
                emit_result(
                    TelemetryOperation::WebhookParse,
                    &result,
                    "router webhook parse completed",
                );
                result
            }
            .instrument(span)
            .await;
        }

        #[cfg(not(feature = "telemetry"))]
        self.parse_webhook_inner(provider, request).await
    }

    async fn parse_webhook_inner(
        &self,
        provider: PaymentProvider,
        request: WebhookRequest<'_>,
    ) -> Result<PaymentEvent, PaymentError> {
        let _ = &request;
        match provider.as_builtin() {
            Some(BuiltinProvider::Stripe) => {
                #[cfg(feature = "stripe")]
                if let Some(connector) = self.stripe.as_ref() {
                    return connector.parse_webhook(request).await;
                }
                Err(Self::not_configured(BuiltinProvider::Stripe))
            }
            Some(BuiltinProvider::PayPal) => {
                #[cfg(feature = "paypal")]
                if let Some(connector) = self.paypal.as_ref() {
                    return connector.parse_webhook(request).await;
                }
                Err(Self::not_configured(BuiltinProvider::PayPal))
            }
            Some(BuiltinProvider::Lipila) => {
                #[cfg(feature = "lipila")]
                if let Some(connector) = self.lipila.as_ref() {
                    return connector.parse_webhook(request).await;
                }
                Err(Self::not_configured(BuiltinProvider::Lipila))
            }
            Some(provider) => Err(Self::not_configured(provider)),
            None => Err(PaymentError::ConnectorNotConfigured { provider }),
        }
    }

    /// Parses a provider fraud or dispute webhook.
    ///
    /// # Errors
    ///
    /// Returns an error when the provider is not configured or parsing fails.
    #[cfg(feature = "fraud")]
    pub async fn parse_fraud_webhook(
        &self,
        provider: PaymentProvider,
        request: WebhookRequest<'_>,
    ) -> Result<FraudEvent, PaymentError> {
        let _ = &request;
        match provider.as_builtin() {
            Some(BuiltinProvider::Stripe) => {
                #[cfg(feature = "stripe")]
                if let Some(connector) = self.stripe.as_ref() {
                    return connector.parse_fraud_webhook(request).await;
                }
                Err(Self::not_configured(BuiltinProvider::Stripe))
            }
            Some(provider) => Err(PaymentError::UnsupportedOperation(format!(
                "{:?} fraud webhooks are not supported",
                PaymentProvider::from(provider)
            ))),
            None => Err(PaymentError::ConnectorNotConfigured { provider }),
        }
    }

    async fn create_payment_with_provider(
        &self,
        provider: BuiltinProvider,
        request: CreatePaymentRequest,
    ) -> Result<PaymentSession, PaymentError> {
        let _ = &request;
        match provider {
            BuiltinProvider::Stripe => {
                #[cfg(feature = "stripe")]
                if let Some(connector) = self.stripe.as_ref() {
                    return connector.create_payment(request).await;
                }
                Err(Self::not_configured(provider))
            }
            BuiltinProvider::PayPal => {
                #[cfg(feature = "paypal")]
                if let Some(connector) = self.paypal.as_ref() {
                    return connector.create_payment(request).await;
                }
                Err(Self::not_configured(provider))
            }
            BuiltinProvider::Lipila => {
                #[cfg(feature = "lipila")]
                if let Some(connector) = self.lipila.as_ref() {
                    return connector.create_payment(request).await;
                }
                Err(Self::not_configured(provider))
            }
            BuiltinProvider::Circle
            | BuiltinProvider::Coinbase
            | BuiltinProvider::Bridge
            | BuiltinProvider::Binance
            | BuiltinProvider::MtnMomo
            | BuiltinProvider::Mpesa
            | BuiltinProvider::AirtelMoney
            | BuiltinProvider::Flutterwave
            | BuiltinProvider::Paystack
            | BuiltinProvider::OrangeMoney => Err(Self::not_configured(provider)),
        }
    }

    fn route_provider(&self, method: &PaymentMethod) -> Result<BuiltinProvider, PaymentError> {
        match method {
            PaymentMethod::Card(_) => Ok(BuiltinProvider::Stripe),
            PaymentMethod::Stablecoin(method) => {
                let asset = method.preferred_asset.as_ref().map(CryptoAsset::from);
                if let Some(provider) = self.route_crypto_provider(asset.as_ref(), None) {
                    return Ok(provider);
                }

                if Self::uses_stripe_default_stablecoin(method.preferred_asset.as_ref()) {
                    return Ok(BuiltinProvider::Stripe);
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
            PaymentMethod::PayPal(_) => Ok(BuiltinProvider::PayPal),
            PaymentMethod::MobileMoney(method) => self
                .mobile_money_routes
                .get(&method.country)
                .copied()
                .ok_or_else(|| PaymentError::UnsupportedPaymentRoute {
                    method: "mobile_money".to_owned(),
                    country: Some(method.country.clone()),
                }),
        }
    }

    fn route_crypto_provider(
        &self,
        asset: Option<&CryptoAsset>,
        network: Option<&CryptoNetwork>,
    ) -> Option<BuiltinProvider> {
        asset
            .zip(network)
            .and_then(|(asset, network)| {
                self.crypto_asset_network_routes
                    .get(asset)
                    .and_then(|routes| routes.get(network))
                    .copied()
            })
            .or_else(|| asset.and_then(|asset| self.crypto_asset_routes.get(asset).copied()))
            .or_else(|| {
                network.and_then(|network| self.crypto_network_routes.get(network).copied())
            })
            .or(self.default_crypto_provider)
    }

    const fn uses_stripe_default_stablecoin(asset: Option<&StablecoinAsset>) -> bool {
        matches!(asset, None | Some(StablecoinAsset::Usdc))
    }

    fn not_configured(provider: BuiltinProvider) -> PaymentError {
        PaymentError::ConnectorNotConfigured {
            provider: PaymentProvider::from(provider),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{CryptoAsset, CryptoNetwork, IdempotencyKey, Money};

    use super::*;

    #[cfg(feature = "fraud")]
    fn rejected_fraud_request() -> CreatePaymentRequest {
        CreatePaymentRequest::builder()
            .amount(Money::new_minor(100, "USD").expect("money should be valid"))
            .reference("ORDER-FRAUD")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::paypal())
            .risk_context(
                crate::RiskContext::new().with_velocity(
                    crate::VelocityRiskContext::new().with_chargebacks_last_90_days(1),
                ),
            )
            .build()
            .expect("request should be valid")
    }

    #[tokio::test]
    async fn routes_card_to_stripe() {
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(100, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::card())
            .build()
            .expect("request should be valid");

        assert!(matches!(
            PaymentRouter::new().create_payment(request).await,
            Err(PaymentError::ConnectorNotConfigured {
                provider: PaymentProvider::Stripe
            })
        ));
    }

    #[cfg(feature = "fraud")]
    #[tokio::test]
    async fn risk_aware_creation_returns_rejected_session_without_provider_io() {
        let policy = FraudPolicy::new().enforce();
        let session = PaymentRouter::new()
            .create_payment_with_risk(rejected_fraud_request(), &policy)
            .await
            .expect("rejected risk-aware payment should not call provider");

        assert!(session.payment().is_none());
        assert_eq!(session.assessment().decision(), crate::RiskDecision::Reject);
    }

    #[cfg(feature = "fraud")]
    #[tokio::test]
    async fn observe_only_rejection_still_executes_provider_path() {
        let policy = FraudPolicy::new().observe_only();

        assert!(matches!(
            PaymentRouter::new()
                .create_payment_with_risk(rejected_fraud_request(), &policy)
                .await,
            Err(PaymentError::ConnectorNotConfigured {
                provider: PaymentProvider::PayPal
            })
        ));
    }

    #[tokio::test]
    async fn mobile_money_route_can_target_modeled_provider() {
        let mut router = PaymentRouter::new();
        router.route_mobile_money(
            CountryCode::new("ZM").expect("country should be valid"),
            BuiltinProvider::MtnMomo,
        );
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
            router.create_payment(request).await,
            Err(PaymentError::ConnectorNotConfigured {
                provider: PaymentProvider::MtnMomo
            })
        ));
    }

    #[tokio::test]
    async fn crypto_route_can_target_wallet_or_crypto_providers() {
        let mut router = PaymentRouter::new();
        router.route_crypto(BuiltinProvider::Coinbase);
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(100, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::crypto(CryptoAsset::Btc))
            .build()
            .expect("request should be valid");

        assert!(matches!(
            router.create_payment(request).await,
            Err(PaymentError::ConnectorNotConfigured {
                provider: PaymentProvider::Coinbase
            })
        ));
    }

    #[tokio::test]
    async fn crypto_asset_network_route_takes_precedence() {
        let mut router = PaymentRouter::new();
        router.route_crypto(BuiltinProvider::Coinbase);
        router.route_crypto_asset_network(
            CryptoAsset::Usdc,
            CryptoNetwork::Base,
            BuiltinProvider::Bridge,
        );
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(100, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::usdc_on(CryptoNetwork::Base))
            .build()
            .expect("request should be valid");

        assert!(matches!(
            router.create_payment(request).await,
            Err(PaymentError::ConnectorNotConfigured {
                provider: PaymentProvider::Bridge
            })
        ));
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
        let mut router = PaymentRouter::new();
        router.route_crypto_asset(CryptoAsset::Usdt, BuiltinProvider::Binance);
        let request = CreatePaymentRequest::builder()
            .amount(Money::new_minor(100, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(PaymentMethod::stablecoin(StablecoinAsset::Usdt))
            .build()
            .expect("request should be valid");

        assert!(matches!(
            router.create_payment(request).await,
            Err(PaymentError::ConnectorNotConfigured {
                provider: PaymentProvider::Binance
            })
        ));
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
    async fn rejects_capture_for_non_capturable_builtin_provider() {
        let reference = ProviderReference::new("provider-ref").expect("reference should be valid");

        assert!(matches!(
            PaymentRouter::new()
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
