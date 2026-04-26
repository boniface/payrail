use async_trait::async_trait;

use crate::{
    CaptureRequest, CaptureResponse, CreatePaymentRequest, PaymentError, PaymentEvent,
    PaymentProvider, PaymentSession, PaymentStatusResponse, ProviderReference, RefundRequest,
    RefundResponse, WebhookRequest,
};

/// Base payment connector operations.
#[async_trait]
pub trait PaymentConnector: Send + Sync {
    /// Returns the provider handled by this connector.
    fn provider(&self) -> PaymentProvider;

    /// Creates a payment.
    async fn create_payment(
        &self,
        request: CreatePaymentRequest,
    ) -> Result<PaymentSession, PaymentError>;

    /// Gets payment status.
    async fn get_payment_status(
        &self,
        provider_reference: &ProviderReference,
    ) -> Result<PaymentStatusResponse, PaymentError>;

    /// Refunds a payment.
    async fn refund_payment(&self, request: RefundRequest) -> Result<RefundResponse, PaymentError>;

    /// Parses a webhook.
    async fn parse_webhook(
        &self,
        request: WebhookRequest<'_>,
    ) -> Result<PaymentEvent, PaymentError>;
}

/// Optional capture capability.
#[async_trait]
pub trait CapturablePaymentConnector: PaymentConnector {
    /// Captures an approved or authorized payment.
    async fn capture_payment(
        &self,
        request: CaptureRequest,
    ) -> Result<CaptureResponse, PaymentError>;
}
