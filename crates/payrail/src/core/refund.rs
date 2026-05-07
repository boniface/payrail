use crate::{IdempotencyKey, Money, PaymentProvider, PaymentStatus, ProviderReference};

/// Refund request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefundRequest {
    /// Provider to route the refund to.
    pub provider: PaymentProvider,
    /// Provider payment reference.
    pub provider_reference: ProviderReference,
    /// Required idempotency key for retry-safe refund creation.
    pub idempotency_key: IdempotencyKey,
    /// Optional partial refund amount.
    pub amount: Option<Money>,
    /// Optional reason.
    pub reason: Option<String>,
}

/// Capture request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureRequest {
    /// Provider to route the capture to.
    pub provider: PaymentProvider,
    /// Provider payment reference.
    pub provider_reference: ProviderReference,
    /// Required idempotency key for retry-safe capture.
    pub idempotency_key: IdempotencyKey,
}

/// Refund response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefundResponse {
    /// Provider handling the refund.
    pub provider: PaymentProvider,
    /// Provider refund reference.
    pub provider_reference: ProviderReference,
    /// Normalized status after refund.
    pub status: PaymentStatus,
}

impl RefundResponse {
    /// Returns the provider handling the refund.
    #[inline]
    #[must_use]
    pub const fn provider(&self) -> &PaymentProvider {
        &self.provider
    }

    /// Returns the provider refund reference.
    #[inline]
    #[must_use]
    pub const fn provider_reference(&self) -> &ProviderReference {
        &self.provider_reference
    }

    /// Returns the normalized refund status.
    #[inline]
    #[must_use]
    pub const fn status(&self) -> PaymentStatus {
        self.status
    }
}

/// Capture response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureResponse {
    /// Provider handling the capture.
    pub provider: PaymentProvider,
    /// Provider payment reference.
    pub provider_reference: ProviderReference,
    /// Normalized status after capture.
    pub status: PaymentStatus,
}

impl CaptureResponse {
    /// Returns the provider handling the capture.
    #[inline]
    #[must_use]
    pub const fn provider(&self) -> &PaymentProvider {
        &self.provider
    }

    /// Returns the provider payment reference.
    #[inline]
    #[must_use]
    pub const fn provider_reference(&self) -> &ProviderReference {
        &self.provider_reference
    }

    /// Returns the normalized capture status.
    #[inline]
    #[must_use]
    pub const fn status(&self) -> PaymentStatus {
        self.status
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_requests_carry_idempotency_keys() {
        let key = IdempotencyKey::new("ORDER-1:refund").expect("key should be valid");
        let refund = RefundRequest {
            provider: PaymentProvider::Stripe,
            provider_reference: ProviderReference::new("pi_123")
                .expect("reference should be valid"),
            idempotency_key: key.clone(),
            amount: None,
            reason: None,
        };
        let capture = CaptureRequest {
            provider: PaymentProvider::PayPal,
            provider_reference: ProviderReference::new("ORDER-1")
                .expect("reference should be valid"),
            idempotency_key: IdempotencyKey::new("ORDER-1:capture").expect("key should be valid"),
        };

        assert_eq!(refund.idempotency_key, key);
        assert_eq!(capture.provider, PaymentProvider::PayPal);
    }

    #[test]
    fn refund_response_accessors_return_normalized_fields() {
        let response = RefundResponse {
            provider: PaymentProvider::Stripe,
            provider_reference: ProviderReference::new("re_123")
                .expect("reference should be valid"),
            status: PaymentStatus::Refunded,
        };

        assert_eq!(response.provider(), &PaymentProvider::Stripe);
        assert_eq!(response.provider_reference().as_str(), "re_123");
        assert_eq!(response.status(), PaymentStatus::Refunded);
    }

    #[test]
    fn capture_response_accessors_return_normalized_fields() {
        let response = CaptureResponse {
            provider: PaymentProvider::PayPal,
            provider_reference: ProviderReference::new("ORDER-1")
                .expect("reference should be valid"),
            status: PaymentStatus::Succeeded,
        };

        assert_eq!(response.provider(), &PaymentProvider::PayPal);
        assert_eq!(response.provider_reference().as_str(), "ORDER-1");
        assert_eq!(response.status(), PaymentStatus::Succeeded);
    }
}
