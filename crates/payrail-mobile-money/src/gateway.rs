use payrail_core::{
    CreatePaymentRequest, PaymentError, PaymentSession, PaymentStatusResponse, ProviderReference,
};

/// Shared trait for Mobile Money gateways.
#[async_trait::async_trait]
pub trait MobileMoneyGateway: Send + Sync {
    /// Creates a Mobile Money collection.
    async fn create_collection(
        &self,
        request: CreatePaymentRequest,
    ) -> Result<PaymentSession, PaymentError>;

    /// Gets Mobile Money collection status.
    async fn collection_status(
        &self,
        provider_reference: &ProviderReference,
    ) -> Result<PaymentStatusResponse, PaymentError>;
}
