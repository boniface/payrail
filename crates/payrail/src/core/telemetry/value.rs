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
    use crate::{CountryCode, CurrencyCode, MerchantReference, ProviderErrorDetails};

    use super::*;

    #[test]
    fn provider_names_are_stable_and_low_cardinality() {
        let cases = [
            (PaymentProvider::Stripe, "stripe"),
            (PaymentProvider::PayPal, "paypal"),
            (PaymentProvider::Lipila, "lipila"),
            (PaymentProvider::Circle, "circle"),
            (PaymentProvider::Coinbase, "coinbase"),
            (PaymentProvider::Bridge, "bridge"),
            (PaymentProvider::Binance, "binance"),
            (PaymentProvider::MtnMomo, "mtn_momo"),
            (PaymentProvider::Mpesa, "mpesa"),
            (PaymentProvider::AirtelMoney, "airtel_money"),
            (PaymentProvider::Flutterwave, "flutterwave"),
            (PaymentProvider::Paystack, "paystack"),
            (PaymentProvider::OrangeMoney, "orange_money"),
            (
                PaymentProvider::Other("merchant-specific-provider".to_owned()),
                "other",
            ),
        ];

        for (provider, expected) in cases {
            assert_eq!(provider_name(&provider), expected);
        }
    }

    #[test]
    fn payment_method_kinds_are_stable() {
        let mobile_money = PaymentMethod::mobile_money_zambia("260971234567")
            .expect("mobile money method should be valid");
        let cases = [
            (PaymentMethod::card(), "card"),
            (PaymentMethod::stablecoin_usdc(), "stablecoin"),
            (PaymentMethod::usdc_on(crate::CryptoNetwork::Base), "crypto"),
            (PaymentMethod::paypal(), "paypal"),
            (mobile_money, "mobile_money"),
        ];

        for (method, expected) in cases {
            assert_eq!(payment_method_kind(&method), expected);
        }
    }

    #[test]
    fn checkout_ui_mode_names_are_stable() {
        let cases = [
            (CheckoutUiMode::Hosted, "hosted"),
            (CheckoutUiMode::Custom, "custom"),
            (CheckoutUiMode::Elements, "elements"),
        ];

        for (mode, expected) in cases {
            assert_eq!(checkout_ui_mode_name(mode), expected);
        }
    }

    #[test]
    fn payment_status_names_are_stable() {
        let cases = [
            (PaymentStatus::Created, "created"),
            (PaymentStatus::RequiresAction, "requires_action"),
            (PaymentStatus::Pending, "pending"),
            (PaymentStatus::Processing, "processing"),
            (PaymentStatus::Authorized, "authorized"),
            (PaymentStatus::Succeeded, "succeeded"),
            (PaymentStatus::Failed, "failed"),
            (PaymentStatus::Cancelled, "cancelled"),
            (PaymentStatus::Expired, "expired"),
            (PaymentStatus::Refunded, "refunded"),
            (PaymentStatus::PartiallyRefunded, "partially_refunded"),
        ];

        for (status, expected) in cases {
            assert_eq!(payment_status_name(status), expected);
        }
    }

    #[test]
    fn payment_event_type_names_are_stable() {
        let cases = [
            (PaymentEventType::PaymentCreated, "payment_created"),
            (
                PaymentEventType::PaymentRequiresAction,
                "payment_requires_action",
            ),
            (PaymentEventType::PaymentPending, "payment_pending"),
            (PaymentEventType::PaymentSucceeded, "payment_succeeded"),
            (PaymentEventType::PaymentFailed, "payment_failed"),
            (PaymentEventType::PaymentCancelled, "payment_cancelled"),
            (PaymentEventType::PaymentRefunded, "payment_refunded"),
            (PaymentEventType::RefundCreated, "refund_created"),
            (PaymentEventType::RefundFailed, "refund_failed"),
            (PaymentEventType::DisputeOpened, "dispute_opened"),
            (PaymentEventType::DisputeUpdated, "dispute_updated"),
            (PaymentEventType::DisputeWon, "dispute_won"),
            (PaymentEventType::DisputeLost, "dispute_lost"),
        ];

        for (event_type, expected) in cases {
            assert_eq!(payment_event_type_name(event_type), expected);
        }
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
    fn error_kinds_are_stable_and_redacted() {
        let country = CountryCode::new("ZM").expect("country should be valid");
        let currency = CurrencyCode::new("USD").expect("currency should be valid");
        let http_error = reqwest::Client::new()
            .get("not a url")
            .build()
            .expect_err("invalid URL should fail request build");
        let json_error = serde_json::from_str::<serde_json::Value>("{")
            .expect_err("invalid JSON should fail to parse");
        let cases = [
            (PaymentError::InvalidAmount(-1), "invalid_amount"),
            (
                PaymentError::InvalidCurrencyCode("SECRET".to_owned()),
                "invalid_currency_code",
            ),
            (
                PaymentError::InvalidCountryCode("SECRET".to_owned()),
                "invalid_country_code",
            ),
            (
                PaymentError::InvalidReference("ORDER-SECRET".to_owned()),
                "invalid_reference",
            ),
            #[cfg(feature = "fraud")]
            (PaymentError::InvalidRiskScore(1001), "invalid_risk_score"),
            #[cfg(feature = "fraud")]
            (
                PaymentError::InvalidRiskContext("SECRET".to_owned()),
                "invalid_risk_context",
            ),
            (
                PaymentError::InvalidIdempotencyKey("SECRET".to_owned()),
                "invalid_idempotency_key",
            ),
            (
                PaymentError::InvalidPhoneNumber("260971234567".to_owned()),
                "invalid_phone_number",
            ),
            (
                PaymentError::InvalidUrl("https://merchant.example/secret".to_owned()),
                "invalid_url",
            ),
            (
                PaymentError::MissingRequiredField("merchant_reference"),
                "missing_required_field",
            ),
            (
                PaymentError::InvalidConfiguration("SECRET".to_owned()),
                "invalid_configuration",
            ),
            (
                PaymentError::ConnectorNotConfigured {
                    provider: PaymentProvider::Stripe,
                },
                "connector_not_configured",
            ),
            (
                PaymentError::UnsupportedPaymentMethod("SECRET".to_owned()),
                "unsupported_payment_method",
            ),
            (
                PaymentError::UnsupportedCountry(country),
                "unsupported_country",
            ),
            (
                PaymentError::UnsupportedCurrency(currency),
                "unsupported_currency",
            ),
            (
                PaymentError::UnsupportedPaymentRoute {
                    method: "SECRET".to_owned(),
                    country: None,
                },
                "unsupported_payment_route",
            ),
            (PaymentError::AuthenticationFailed, "authentication_failed"),
            (
                PaymentError::ProviderRequestFailed {
                    provider: PaymentProvider::Stripe,
                    status: 500,
                    message: "SECRET".to_owned(),
                },
                "provider_request_failed",
            ),
            (
                PaymentError::ProviderUnavailable(PaymentProvider::Stripe),
                "provider_unavailable",
            ),
            (
                PaymentError::RateLimited(PaymentProvider::Stripe),
                "rate_limited",
            ),
            (
                PaymentError::WebhookVerificationFailed,
                "webhook_verification_failed",
            ),
            (
                PaymentError::InvalidWebhookPayload("SECRET".to_owned()),
                "invalid_webhook_payload",
            ),
            (
                PaymentError::UnsupportedOperation("SECRET".to_owned()),
                "unsupported_operation",
            ),
            (PaymentError::Http(http_error), "http"),
            (PaymentError::Json(json_error), "json"),
        ];

        for (error, expected) in cases {
            assert_eq!(error_kind(&error), expected);
        }
    }

    #[test]
    fn references_are_not_needed_for_telemetry_names() {
        let reference = MerchantReference::new("ORDER-1").expect("reference should be valid");

        assert_eq!(reference.as_str(), "ORDER-1");
        assert_eq!(provider_name(&PaymentProvider::Stripe), "stripe");
    }
}
