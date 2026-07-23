use crate::{
    CheckoutUiMode, PaymentError, PaymentEventType, PaymentMethod, PaymentProvider, PaymentStatus,
};

/// Returns a low-cardinality provider name.
#[inline]
#[must_use]
pub fn provider_name(provider: &PaymentProvider) -> &'static str {
    match provider {
        PaymentProvider::Stripe => "stripe",
        PaymentProvider::PayPal => "paypal",
        PaymentProvider::Lipila => "lipila",
        PaymentProvider::Circle => "circle",
        PaymentProvider::Coinbase => "coinbase",
        PaymentProvider::Bridge => "bridge",
        PaymentProvider::Binance => "binance",
        PaymentProvider::MtnMomo => "mtn_momo",
        PaymentProvider::Mpesa => "mpesa",
        PaymentProvider::AirtelMoney => "airtel_money",
        PaymentProvider::Flutterwave => "flutterwave",
        PaymentProvider::Paystack => "paystack",
        PaymentProvider::OrangeMoney => "orange_money",
        PaymentProvider::Other(_) => "other",
    }
}

/// Returns a low-cardinality payment method category.
#[inline]
#[must_use]
pub const fn payment_method_kind(method: &PaymentMethod) -> &'static str {
    match method {
        PaymentMethod::Card(_) => "card",
        PaymentMethod::Stablecoin(_) => "stablecoin",
        PaymentMethod::Crypto(_) => "crypto",
        PaymentMethod::PayPal(_) => "paypal",
        PaymentMethod::MobileMoney(_) => "mobile_money",
    }
}

/// Returns a low-cardinality checkout UI mode.
#[inline]
#[must_use]
pub const fn checkout_ui_mode_name(mode: CheckoutUiMode) -> &'static str {
    match mode {
        CheckoutUiMode::Hosted => "hosted",
        CheckoutUiMode::Custom => "custom",
        CheckoutUiMode::Elements => "elements",
    }
}

/// Returns a low-cardinality payment status name.
#[inline]
#[must_use]
pub const fn payment_status_name(status: PaymentStatus) -> &'static str {
    match status {
        PaymentStatus::Created => "created",
        PaymentStatus::RequiresAction => "requires_action",
        PaymentStatus::Pending => "pending",
        PaymentStatus::Processing => "processing",
        PaymentStatus::Authorized => "authorized",
        PaymentStatus::Succeeded => "succeeded",
        PaymentStatus::Failed => "failed",
        PaymentStatus::Cancelled => "cancelled",
        PaymentStatus::Expired => "expired",
        PaymentStatus::Refunded => "refunded",
        PaymentStatus::PartiallyRefunded => "partially_refunded",
    }
}

/// Returns a low-cardinality payment event type name.
#[inline]
#[must_use]
pub const fn payment_event_type_name(event_type: PaymentEventType) -> &'static str {
    match event_type {
        PaymentEventType::PaymentCreated => "payment_created",
        PaymentEventType::PaymentRequiresAction => "payment_requires_action",
        PaymentEventType::PaymentPending => "payment_pending",
        PaymentEventType::PaymentSucceeded => "payment_succeeded",
        PaymentEventType::PaymentFailed => "payment_failed",
        PaymentEventType::PaymentCancelled => "payment_cancelled",
        PaymentEventType::PaymentRefunded => "payment_refunded",
        PaymentEventType::RefundCreated => "refund_created",
        PaymentEventType::RefundFailed => "refund_failed",
        PaymentEventType::DisputeOpened => "dispute_opened",
        PaymentEventType::DisputeUpdated => "dispute_updated",
        PaymentEventType::DisputeWon => "dispute_won",
        PaymentEventType::DisputeLost => "dispute_lost",
    }
}

/// Returns a low-cardinality error kind.
#[inline]
#[must_use]
pub const fn error_kind(error: &PaymentError) -> &'static str {
    match error {
        PaymentError::InvalidAmount(_) => "invalid_amount",
        PaymentError::InvalidCurrencyCode(_) => "invalid_currency_code",
        PaymentError::InvalidCountryCode(_) => "invalid_country_code",
        PaymentError::InvalidReference(_) => "invalid_reference",
        #[cfg(feature = "fraud")]
        PaymentError::InvalidRiskScore(_) => "invalid_risk_score",
        #[cfg(feature = "fraud")]
        PaymentError::InvalidRiskContext(_) => "invalid_risk_context",
        PaymentError::InvalidIdempotencyKey(_) => "invalid_idempotency_key",
        PaymentError::InvalidPhoneNumber(_) => "invalid_phone_number",
        PaymentError::InvalidUrl(_) => "invalid_url",
        PaymentError::MissingRequiredField(_) => "missing_required_field",
        PaymentError::InvalidConfiguration(_) => "invalid_configuration",
        PaymentError::ConnectorNotConfigured { .. } => "connector_not_configured",
        PaymentError::UnsupportedPaymentMethod(_) => "unsupported_payment_method",
        PaymentError::UnsupportedCountry(_) => "unsupported_country",
        PaymentError::UnsupportedCurrency(_) => "unsupported_currency",
        PaymentError::UnsupportedPaymentRoute { .. } => "unsupported_payment_route",
        PaymentError::AuthenticationFailed => "authentication_failed",
        PaymentError::ProviderRequestFailed { .. } => "provider_request_failed",
        PaymentError::ProviderDetails { .. } => "provider_details",
        PaymentError::ProviderUnavailable(_) => "provider_unavailable",
        PaymentError::RateLimited(_) => "rate_limited",
        PaymentError::WebhookVerificationFailed => "webhook_verification_failed",
        PaymentError::InvalidWebhookPayload(_) => "invalid_webhook_payload",
        PaymentError::UnsupportedOperation(_) => "unsupported_operation",
        PaymentError::Http(_) => "http",
        PaymentError::Json(_) => "json",
    }
}

#[cfg(test)]
mod tests {
    use crate::{MerchantReference, ProviderErrorDetails};

    use super::*;

    #[test]
    fn provider_other_is_low_cardinality() {
        let provider = PaymentProvider::Other("merchant-specific-provider".to_owned());

        assert_eq!(provider_name(&provider), "other");
    }

    #[test]
    fn payment_method_kind_is_low_cardinality() {
        assert_eq!(payment_method_kind(&PaymentMethod::card()), "card");
        assert_eq!(payment_method_kind(&PaymentMethod::paypal()), "paypal");
    }

    #[test]
    fn error_kind_does_not_expose_error_values() {
        let error = PaymentError::InvalidReference("ORDER-SECRET".to_owned());

        assert_eq!(error_kind(&error), "invalid_reference");
        assert!(!error_kind(&error).contains("ORDER-SECRET"));
    }

    #[test]
    fn provider_details_error_kind_is_redacted() {
        let error = PaymentError::ProviderDetails {
            details: ProviderErrorDetails {
                provider: PaymentProvider::Other("custom-provider".to_owned()),
                status: 500,
                code: Some("raw-code".to_owned()),
                request_id: Some("req_secret".to_owned()),
                message: "provider message".to_owned(),
            },
        };

        assert_eq!(error_kind(&error), "provider_details");
    }

    #[test]
    fn event_type_names_are_stable() {
        assert_eq!(
            payment_event_type_name(PaymentEventType::PaymentSucceeded),
            "payment_succeeded"
        );
        assert_eq!(
            payment_status_name(PaymentStatus::PartiallyRefunded),
            "partially_refunded"
        );
        assert_eq!(checkout_ui_mode_name(CheckoutUiMode::Hosted), "hosted");
    }

    #[test]
    fn references_are_not_needed_for_telemetry_names() {
        let reference = MerchantReference::new("ORDER-1").expect("reference should be valid");

        assert_eq!(reference.as_str(), "ORDER-1");
        assert_eq!(provider_name(&PaymentProvider::Stripe), "stripe");
    }
}
