use crate::{
    CaptureRequest, CaptureResponse, CreatePaymentRequest, PaymentError, PaymentEvent,
    PaymentProvider, PaymentSession, PaymentStatusResponse, ProviderReference, RefundRequest,
    RefundResponse, WebhookRequest,
};

use crate::PaymentRouter;

/// PayRail facade client.
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
    use std::sync::Arc;

    use crate::{
        CaptureResponse, IdempotencyKey, MerchantReference, PaymentConnector, PaymentEventType,
        PaymentStatus,
    };
    use async_trait::async_trait;
    use http::HeaderMap;

    use super::*;

    #[derive(Debug)]
    struct ClientConnector;

    #[async_trait]
    impl PaymentConnector for ClientConnector {
        fn provider(&self) -> PaymentProvider {
            PaymentProvider::PayPal
        }

        async fn create_payment(
            &self,
            request: CreatePaymentRequest,
        ) -> Result<PaymentSession, PaymentError> {
            PaymentSession::new(
                PaymentProvider::PayPal,
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
                provider: PaymentProvider::PayPal,
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
                provider: PaymentProvider::PayPal,
                provider_reference: ProviderReference::new("provider-ref")?,
                merchant_reference: Some(MerchantReference::new("ORDER-1")?),
                status: PaymentStatus::Succeeded,
                amount: None,
                event_type: PaymentEventType::PaymentSucceeded,
                message: None,
            })
        }
    }

    #[async_trait]
    impl crate::CapturablePaymentConnector for ClientConnector {
        async fn capture_payment(
            &self,
            request: CaptureRequest,
        ) -> Result<CaptureResponse, PaymentError> {
            Ok(CaptureResponse {
                provider: PaymentProvider::PayPal,
                provider_reference: request.provider_reference,
                status: PaymentStatus::Succeeded,
            })
        }
    }

    #[tokio::test]
    async fn client_delegates_all_operations() {
        let connector = Arc::new(ClientConnector);
        let mut router = PaymentRouter::new();
        let payment_connector: Arc<dyn PaymentConnector> = connector.clone();
        let capturable: Arc<dyn crate::CapturablePaymentConnector> = connector;
        router.register_capturable(payment_connector, capturable);
        let client = PayRailClient::new(router);
        let provider_reference =
            ProviderReference::new("provider-ref").expect("reference should be valid");
        let request = CreatePaymentRequest::builder()
            .amount(crate::Money::new_minor(1_000, "USD").expect("money should be valid"))
            .reference("ORDER-1")
            .expect("reference should be valid")
            .payment_method(crate::PaymentMethod::paypal())
            .build()
            .expect("request should be valid");

        let session = client
            .create_payment(request)
            .await
            .expect("payment should create");
        let status = client
            .get_payment_status(PaymentProvider::PayPal, &provider_reference)
            .await
            .expect("status should route");
        let refund = client
            .refund_payment(RefundRequest {
                provider: PaymentProvider::PayPal,
                provider_reference: provider_reference.clone(),
                idempotency_key: IdempotencyKey::new("ORDER-1:refund")
                    .expect("key should be valid"),
                amount: None,
                reason: None,
            })
            .await
            .expect("refund should route");
        let capture = client
            .capture_payment(CaptureRequest {
                provider: PaymentProvider::PayPal,
                provider_reference: provider_reference.clone(),
                idempotency_key: IdempotencyKey::new("ORDER-1:capture")
                    .expect("key should be valid"),
            })
            .await
            .expect("capture should route");
        let event = client
            .parse_webhook(
                PaymentProvider::PayPal,
                WebhookRequest::new(b"{}", HeaderMap::new()),
            )
            .await
            .expect("webhook should route");

        assert_eq!(session.status, PaymentStatus::Created);
        assert_eq!(status.status, PaymentStatus::Created);
        assert_eq!(refund.status, PaymentStatus::Refunded);
        assert_eq!(capture.status, PaymentStatus::Succeeded);
        assert_eq!(event.status, PaymentStatus::Succeeded);
    }
}
