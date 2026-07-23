use crate::{
    CaptureRequest, CaptureResponse, CreatePaymentRequest, PaymentError, PaymentEvent,
    PaymentProvider, PaymentSession, PaymentStatusResponse, ProviderReference, RefundRequest,
    RefundResponse, WebhookRequest,
};

#[cfg(feature = "fraud")]
use crate::{FraudEvent, FraudPolicy, RiskAssessment, RiskAwarePaymentSession};
#[cfg(feature = "telemetry")]
use crate::{
    TelemetryOperation, emit_result, payment_method_kind, payment_status_name, provider_name,
};

use crate::PaymentRouter;
#[cfg(feature = "telemetry")]
use tracing::Instrument;

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
        #[cfg(feature = "telemetry")]
        {
            let span = tracing::info_span!(
                "payrail.payment.create",
                "payrail.operation" = TelemetryOperation::PaymentCreate.as_str(),
                "payrail.payment_method" = payment_method_kind(request.payment_method()),
                "payrail.checkout_ui_mode" =
                    crate::checkout_ui_mode_name(request.checkout_ui_mode()),
                "payrail.has_idempotency_key" = request.idempotency_key().is_some(),
                "payrail.has_callback_url" = request.callback_url().is_some()
            );
            return async {
                let result = self.router.create_payment(request).await;
                emit_result(
                    TelemetryOperation::PaymentCreate,
                    &result,
                    "payment create completed",
                );
                result
            }
            .instrument(span)
            .await;
        }

        #[cfg(not(feature = "telemetry"))]
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
        #[cfg(feature = "telemetry")]
        {
            let span = tracing::info_span!(
                "payrail.payment.status",
                "payrail.operation" = TelemetryOperation::PaymentStatus.as_str(),
                "payrail.provider" = provider_name(&provider)
            );
            return async {
                let result = self
                    .router
                    .get_payment_status(provider, provider_reference)
                    .await;
                if let Ok(status) = result.as_ref() {
                    tracing::debug!(
                        "payrail.status" = payment_status_name(status.status()),
                        "payment status normalized"
                    );
                }
                emit_result(
                    TelemetryOperation::PaymentStatus,
                    &result,
                    "payment status completed",
                );
                result
            }
            .instrument(span)
            .await;
        }

        #[cfg(not(feature = "telemetry"))]
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
        #[cfg(feature = "telemetry")]
        {
            let span = tracing::info_span!(
                "payrail.payment.refund",
                "payrail.operation" = TelemetryOperation::PaymentRefund.as_str(),
                "payrail.provider" = provider_name(&request.provider)
            );
            return async {
                let result = self.router.refund_payment(request).await;
                emit_result(
                    TelemetryOperation::PaymentRefund,
                    &result,
                    "payment refund completed",
                );
                result
            }
            .instrument(span)
            .await;
        }

        #[cfg(not(feature = "telemetry"))]
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
        #[cfg(feature = "telemetry")]
        {
            let span = tracing::info_span!(
                "payrail.payment.capture",
                "payrail.operation" = TelemetryOperation::PaymentCapture.as_str(),
                "payrail.provider" = provider_name(&request.provider)
            );
            return async {
                let result = self.router.capture_payment(request).await;
                emit_result(
                    TelemetryOperation::PaymentCapture,
                    &result,
                    "payment capture completed",
                );
                result
            }
            .instrument(span)
            .await;
        }

        #[cfg(not(feature = "telemetry"))]
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
        #[cfg(feature = "telemetry")]
        {
            let span = tracing::info_span!(
                "payrail.webhook.parse",
                "payrail.operation" = TelemetryOperation::WebhookParse.as_str(),
                "payrail.provider" = provider_name(&provider),
                "payrail.payload_len" = request.payload.len()
            );
            return async {
                let result = self.router.parse_webhook(provider, request).await;
                if let Ok(event) = result.as_ref() {
                    tracing::debug!(
                        "payrail.event_type" = crate::payment_event_type_name(event.event_type()),
                        "payrail.status" = payment_status_name(event.status()),
                        "webhook event normalized"
                    );
                }
                emit_result(
                    TelemetryOperation::WebhookParse,
                    &result,
                    "webhook parse completed",
                );
                result
            }
            .instrument(span)
            .await;
        }

        #[cfg(not(feature = "telemetry"))]
        self.router.parse_webhook(provider, request).await
    }

    /// Parses a provider fraud or dispute webhook.
    ///
    /// # Errors
    ///
    /// Returns an error when routing, verification, or parsing fails.
    #[cfg(feature = "fraud")]
    pub async fn parse_fraud_webhook(
        &self,
        provider: PaymentProvider,
        request: WebhookRequest<'_>,
    ) -> Result<FraudEvent, PaymentError> {
        self.router.parse_fraud_webhook(provider, request).await
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
