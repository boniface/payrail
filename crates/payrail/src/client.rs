use crate::{
    CaptureRequest, CaptureResponse, CreatePaymentRequest, PaymentError, PaymentEvent,
    PaymentProvider, PaymentSession, PaymentStatusResponse, ProviderReference, RefundRequest,
    RefundResponse, WebhookRequest,
};

#[cfg(feature = "fraud")]
use crate::{FraudPolicy, RiskAssessment, RiskAwarePaymentSession};

use crate::PaymentRouter;

/// `PayRail` facade client.
#[derive(Debug, Clone)]
pub struct PayRailClient {
    router: PaymentRouter,
}

impl PayRailClient {
    /// Creates a client from a router.
    #[inline]
    #[must_use]
    pub const fn new(router: PaymentRouter) -> Self {
        Self { router }
    }

    /// Creates a payment.
    ///
    /// # Errors
    ///
    /// Returns an error when routing or provider execution fails.
    pub async fn create_payment(
        &self,
        request: CreatePaymentRequest,
    ) -> Result<PaymentSession, PaymentError> {
        self.router.create_payment(request).await
    }

    /// Assesses payment risk using the default local fraud policy.
    #[cfg(feature = "fraud")]
    #[inline]
    #[must_use]
    pub fn assess_payment_risk(&self, request: &CreatePaymentRequest) -> RiskAssessment {
        self.router.assess_payment_risk(request)
    }

    /// Assesses risk and creates a payment only when the policy allows provider execution.
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
        self.router.create_payment_with_risk(request, policy).await
    }

    /// Gets payment status.
    ///
    /// # Errors
    ///
    /// Returns an error when routing or provider execution fails.
    pub async fn get_payment_status(
        &self,
        provider: PaymentProvider,
        provider_reference: &ProviderReference,
    ) -> Result<PaymentStatusResponse, PaymentError> {
        self.router
            .get_payment_status(provider, provider_reference)
            .await
    }

    /// Refunds a payment.
    ///
    /// # Errors
    ///
    /// Returns an error when routing or provider execution fails.
    pub async fn refund_payment(
        &self,
        request: RefundRequest,
    ) -> Result<RefundResponse, PaymentError> {
        self.router.refund_payment(request).await
    }

    /// Captures a payment.
    ///
    /// # Errors
    ///
    /// Returns an error when the provider is not capture-capable or execution fails.
    pub async fn capture_payment(
        &self,
        request: CaptureRequest,
    ) -> Result<CaptureResponse, PaymentError> {
        self.router.capture_payment(request).await
    }

    /// Parses a webhook.
    ///
    /// # Errors
    ///
    /// Returns an error when routing, verification, or parsing fails.
    pub async fn parse_webhook(
        &self,
        provider: PaymentProvider,
        request: WebhookRequest<'_>,
    ) -> Result<PaymentEvent, PaymentError> {
        self.router.parse_webhook(provider, request).await
    }
}

#[cfg(test)]
mod tests {
    use crate::IdempotencyKey;
    use http::HeaderMap;

    use super::*;

    #[tokio::test]
    async fn client_delegates_to_static_router_errors() {
        let client = PayRailClient::new(PaymentRouter::new());
        let provider_reference =
            ProviderReference::new("provider-ref").expect("reference should be valid");
        let request = CreatePaymentRequest::builder()
            .amount(crate::Money::new_minor(1_000, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(crate::PaymentMethod::paypal())
            .build()
            .expect("request should be valid");

        assert!(matches!(
            client.create_payment(request).await,
            Err(PaymentError::ConnectorNotConfigured {
                provider: PaymentProvider::PayPal
            })
        ));
        assert!(matches!(
            client
                .get_payment_status(PaymentProvider::PayPal, &provider_reference)
                .await,
            Err(PaymentError::ConnectorNotConfigured {
                provider: PaymentProvider::PayPal
            })
        ));
        assert!(matches!(
            client
                .refund_payment(RefundRequest {
                    provider: PaymentProvider::PayPal,
                    provider_reference: provider_reference.clone(),
                    idempotency_key: IdempotencyKey::new("ORDER-1:refund")
                        .expect("key should be valid"),
                    amount: None,
                    reason: None,
                })
                .await,
            Err(PaymentError::ConnectorNotConfigured {
                provider: PaymentProvider::PayPal
            })
        ));
        assert!(matches!(
            client
                .capture_payment(CaptureRequest {
                    provider: PaymentProvider::PayPal,
                    provider_reference: provider_reference.clone(),
                    idempotency_key: IdempotencyKey::new("ORDER-1:capture")
                        .expect("key should be valid"),
                })
                .await,
            Err(PaymentError::ConnectorNotConfigured {
                provider: PaymentProvider::PayPal
            })
        ));
        assert!(matches!(
            client
                .parse_webhook(
                    PaymentProvider::PayPal,
                    WebhookRequest::new(b"{}", HeaderMap::new()),
                )
                .await,
            Err(PaymentError::ConnectorNotConfigured {
                provider: PaymentProvider::PayPal
            })
        ));
    }

    #[cfg(feature = "fraud")]
    #[tokio::test]
    async fn client_delegates_risk_aware_payment_creation() {
        let client = PayRailClient::new(PaymentRouter::new());
        let request =
            CreatePaymentRequest::builder()
                .amount(crate::Money::new_minor(1_000, "USD").expect("money should be valid"))
                .reference("ORDER-FRAUD")
                .expect("reference should be valid")
                .payment_method(crate::PaymentMethod::paypal())
                .risk_context(crate::RiskContext::new().with_velocity(
                    crate::VelocityRiskContext::new().with_chargebacks_last_90_days(1),
                ))
                .build()
                .expect("request should be valid");

        let result = client
            .create_payment_with_risk(request, &FraudPolicy::new().enforce())
            .await
            .expect("enforced fraud rejection should return a risk-aware result");

        assert!(result.payment().is_none());
        assert_eq!(result.assessment().decision(), crate::RiskDecision::Reject);
    }
}
