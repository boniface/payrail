use std::fmt;

use crate::{CountryCode, PaymentError, VerificationStatus};

/// Customer segment supplied by the application.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CustomerSegment {
    /// New customer.
    New,
    /// Returning customer.
    Returning,
    /// Trusted customer.
    Trusted,
    /// High-risk customer segment.
    HighRisk,
    /// Segment not modeled directly.
    Other(String),
}

/// Merchant vertical supplied by the application.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MerchantVertical {
    /// Digital goods.
    DigitalGoods,
    /// Marketplace.
    Marketplace,
    /// Travel.
    Travel,
    /// Gaming.
    Gaming,
    /// Financial services.
    FinancialServices,
    /// Vertical not modeled directly.
    Other(String),
}

/// Pseudonymous device identifier.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DeviceId(String);

impl DeviceId {
    /// Creates a validated pseudonymous device identifier.
    ///
    /// # Errors
    ///
    /// Returns an error when the identifier is empty or longer than 255 bytes.
    pub fn new(value: impl AsRef<str>) -> Result<Self, PaymentError> {
        validated_token(value.as_ref()).map(Self)
    }

    /// Returns the pseudonymous identifier.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for DeviceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DeviceId([redacted])")
    }
}

/// Provider-issued device token.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DeviceProviderToken(String);

impl DeviceProviderToken {
    /// Creates a validated provider device token.
    ///
    /// # Errors
    ///
    /// Returns an error when the token is empty or longer than 255 bytes.
    pub fn new(value: impl AsRef<str>) -> Result<Self, PaymentError> {
        validated_token(value.as_ref()).map(Self)
    }

    /// Returns the provider token.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for DeviceProviderToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DeviceProviderToken([redacted])")
    }
}

/// Device risk context supplied by the application or a device provider.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeviceRiskContext {
    device_id: Option<DeviceId>,
    provider_token: Option<DeviceProviderToken>,
    seen_before: Option<bool>,
    recently_changed: Option<bool>,
}

impl DeviceRiskContext {
    /// Creates empty device risk context.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the pseudonymous device identifier.
    #[must_use]
    pub fn with_device_id(mut self, device_id: DeviceId) -> Self {
        self.device_id = Some(device_id);
        self
    }

    /// Sets the provider-issued device token.
    #[must_use]
    pub fn with_provider_token(mut self, provider_token: DeviceProviderToken) -> Self {
        self.provider_token = Some(provider_token);
        self
    }

    /// Sets whether this device has been seen before.
    #[must_use]
    pub const fn with_seen_before(mut self, seen_before: bool) -> Self {
        self.seen_before = Some(seen_before);
        self
    }

    /// Sets whether this device recently changed.
    #[must_use]
    pub const fn with_recently_changed(mut self, recently_changed: bool) -> Self {
        self.recently_changed = Some(recently_changed);
        self
    }

    /// Returns the pseudonymous device identifier.
    #[inline]
    #[must_use]
    pub const fn device_id(&self) -> Option<&DeviceId> {
        self.device_id.as_ref()
    }

    /// Returns the provider-issued device token.
    #[inline]
    #[must_use]
    pub const fn provider_token(&self) -> Option<&DeviceProviderToken> {
        self.provider_token.as_ref()
    }

    /// Returns whether this device has been seen before.
    #[inline]
    #[must_use]
    pub const fn seen_before(&self) -> Option<bool> {
        self.seen_before
    }

    /// Returns whether this device recently changed.
    #[inline]
    #[must_use]
    pub const fn recently_changed(&self) -> Option<bool> {
        self.recently_changed
    }
}

/// Network risk context supplied by the application.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetworkRiskContext {
    ip_country: Option<CountryCode>,
    billing_country: Option<CountryCode>,
    shipping_country: Option<CountryCode>,
    proxy_detected: Option<bool>,
    vpn_detected: Option<bool>,
    tor_detected: Option<bool>,
}

impl NetworkRiskContext {
    /// Creates empty network risk context.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the IP-derived country.
    #[must_use]
    pub fn with_ip_country(mut self, country: CountryCode) -> Self {
        self.ip_country = Some(country);
        self
    }

    /// Sets the billing country.
    #[must_use]
    pub fn with_billing_country(mut self, country: CountryCode) -> Self {
        self.billing_country = Some(country);
        self
    }

    /// Sets the shipping country.
    #[must_use]
    pub fn with_shipping_country(mut self, country: CountryCode) -> Self {
        self.shipping_country = Some(country);
        self
    }

    /// Sets whether a proxy was detected.
    #[must_use]
    pub const fn with_proxy_detected(mut self, detected: bool) -> Self {
        self.proxy_detected = Some(detected);
        self
    }

    /// Sets whether a VPN was detected.
    #[must_use]
    pub const fn with_vpn_detected(mut self, detected: bool) -> Self {
        self.vpn_detected = Some(detected);
        self
    }

    /// Sets whether Tor was detected.
    #[must_use]
    pub const fn with_tor_detected(mut self, detected: bool) -> Self {
        self.tor_detected = Some(detected);
        self
    }

    /// Returns the IP-derived country.
    #[inline]
    #[must_use]
    pub const fn ip_country(&self) -> Option<&CountryCode> {
        self.ip_country.as_ref()
    }

    /// Returns the billing country.
    #[inline]
    #[must_use]
    pub const fn billing_country(&self) -> Option<&CountryCode> {
        self.billing_country.as_ref()
    }

    /// Returns the shipping country.
    #[inline]
    #[must_use]
    pub const fn shipping_country(&self) -> Option<&CountryCode> {
        self.shipping_country.as_ref()
    }

    /// Returns whether a proxy was detected.
    #[inline]
    #[must_use]
    pub const fn proxy_detected(&self) -> Option<bool> {
        self.proxy_detected
    }

    /// Returns whether a VPN was detected.
    #[inline]
    #[must_use]
    pub const fn vpn_detected(&self) -> Option<bool> {
        self.vpn_detected
    }

    /// Returns whether Tor was detected.
    #[inline]
    #[must_use]
    pub const fn tor_detected(&self) -> Option<bool> {
        self.tor_detected
    }
}

/// Velocity risk context supplied by the application.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VelocityRiskContext {
    attempts_last_10_minutes: Option<u16>,
    attempts_last_hour: Option<u16>,
    attempts_last_day: Option<u16>,
    successful_payments_last_day: Option<u16>,
    failed_payments_last_day: Option<u16>,
    chargebacks_last_90_days: Option<u16>,
}

impl VelocityRiskContext {
    /// Creates empty velocity risk context.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets attempts in the last ten minutes.
    #[must_use]
    pub const fn with_attempts_last_10_minutes(mut self, attempts: u16) -> Self {
        self.attempts_last_10_minutes = Some(attempts);
        self
    }

    /// Sets attempts in the last hour.
    #[must_use]
    pub const fn with_attempts_last_hour(mut self, attempts: u16) -> Self {
        self.attempts_last_hour = Some(attempts);
        self
    }

    /// Sets attempts in the last day.
    #[must_use]
    pub const fn with_attempts_last_day(mut self, attempts: u16) -> Self {
        self.attempts_last_day = Some(attempts);
        self
    }

    /// Sets successful payments in the last day.
    #[must_use]
    pub const fn with_successful_payments_last_day(mut self, payments: u16) -> Self {
        self.successful_payments_last_day = Some(payments);
        self
    }

    /// Sets failed payments in the last day.
    #[must_use]
    pub const fn with_failed_payments_last_day(mut self, payments: u16) -> Self {
        self.failed_payments_last_day = Some(payments);
        self
    }

    /// Sets chargebacks in the last ninety days.
    #[must_use]
    pub const fn with_chargebacks_last_90_days(mut self, chargebacks: u16) -> Self {
        self.chargebacks_last_90_days = Some(chargebacks);
        self
    }

    /// Returns attempts in the last ten minutes.
    #[inline]
    #[must_use]
    pub const fn attempts_last_10_minutes(&self) -> Option<u16> {
        self.attempts_last_10_minutes
    }

    /// Returns attempts in the last hour.
    #[inline]
    #[must_use]
    pub const fn attempts_last_hour(&self) -> Option<u16> {
        self.attempts_last_hour
    }

    /// Returns attempts in the last day.
    #[inline]
    #[must_use]
    pub const fn attempts_last_day(&self) -> Option<u16> {
        self.attempts_last_day
    }

    /// Returns successful payments in the last day.
    #[inline]
    #[must_use]
    pub const fn successful_payments_last_day(&self) -> Option<u16> {
        self.successful_payments_last_day
    }

    /// Returns failed payments in the last day.
    #[inline]
    #[must_use]
    pub const fn failed_payments_last_day(&self) -> Option<u16> {
        self.failed_payments_last_day
    }

    /// Returns chargebacks in the last ninety days.
    #[inline]
    #[must_use]
    pub const fn chargebacks_last_90_days(&self) -> Option<u16> {
        self.chargebacks_last_90_days
    }
}

/// Fraud risk context supplied by the application.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RiskContext {
    customer_segment: Option<CustomerSegment>,
    merchant_vertical: Option<MerchantVertical>,
    account_age_days: Option<u32>,
    email_status: VerificationStatus,
    phone_status: VerificationStatus,
    identity_status: VerificationStatus,
    device: Option<DeviceRiskContext>,
    network: Option<NetworkRiskContext>,
    velocity: Option<VelocityRiskContext>,
}

impl RiskContext {
    /// Creates empty risk context.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the customer segment.
    #[must_use]
    pub fn with_customer_segment(mut self, segment: CustomerSegment) -> Self {
        self.customer_segment = Some(segment);
        self
    }

    /// Sets the merchant vertical.
    #[must_use]
    pub fn with_merchant_vertical(mut self, vertical: MerchantVertical) -> Self {
        self.merchant_vertical = Some(vertical);
        self
    }

    /// Sets the customer account age in days.
    #[must_use]
    pub const fn with_account_age_days(mut self, days: u32) -> Self {
        self.account_age_days = Some(days);
        self
    }

    /// Sets the email verification status.
    #[must_use]
    pub const fn with_email_status(mut self, status: VerificationStatus) -> Self {
        self.email_status = status;
        self
    }

    /// Sets the phone verification status.
    #[must_use]
    pub const fn with_phone_status(mut self, status: VerificationStatus) -> Self {
        self.phone_status = status;
        self
    }

    /// Sets the identity verification status.
    #[must_use]
    pub const fn with_identity_status(mut self, status: VerificationStatus) -> Self {
        self.identity_status = status;
        self
    }

    /// Sets the device context.
    #[must_use]
    pub fn with_device(mut self, device: DeviceRiskContext) -> Self {
        self.device = Some(device);
        self
    }

    /// Sets the network context.
    #[must_use]
    pub fn with_network(mut self, network: NetworkRiskContext) -> Self {
        self.network = Some(network);
        self
    }

    /// Sets the velocity context.
    #[must_use]
    pub fn with_velocity(mut self, velocity: VelocityRiskContext) -> Self {
        self.velocity = Some(velocity);
        self
    }

    /// Returns the customer segment.
    #[inline]
    #[must_use]
    pub const fn customer_segment(&self) -> Option<&CustomerSegment> {
        self.customer_segment.as_ref()
    }

    /// Returns the merchant vertical.
    #[inline]
    #[must_use]
    pub const fn merchant_vertical(&self) -> Option<&MerchantVertical> {
        self.merchant_vertical.as_ref()
    }

    /// Returns the customer account age in days.
    #[inline]
    #[must_use]
    pub const fn account_age_days(&self) -> Option<u32> {
        self.account_age_days
    }

    /// Returns the email verification status.
    #[inline]
    #[must_use]
    pub const fn email_status(&self) -> VerificationStatus {
        self.email_status
    }

    /// Returns the phone verification status.
    #[inline]
    #[must_use]
    pub const fn phone_status(&self) -> VerificationStatus {
        self.phone_status
    }

    /// Returns the identity verification status.
    #[inline]
    #[must_use]
    pub const fn identity_status(&self) -> VerificationStatus {
        self.identity_status
    }

    /// Returns the device context.
    #[inline]
    #[must_use]
    pub const fn device(&self) -> Option<&DeviceRiskContext> {
        self.device.as_ref()
    }

    /// Returns the network context.
    #[inline]
    #[must_use]
    pub const fn network(&self) -> Option<&NetworkRiskContext> {
        self.network.as_ref()
    }

    /// Returns the velocity context.
    #[inline]
    #[must_use]
    pub const fn velocity(&self) -> Option<&VelocityRiskContext> {
        self.velocity.as_ref()
    }
}

fn validated_token(value: &str) -> Result<String, PaymentError> {
    let value = value.trim();
    if value.is_empty() || value.len() > 255 {
        return Err(PaymentError::InvalidRiskContext(
            "token cannot be empty or longer than 255 bytes".to_owned(),
        ));
    }

    Ok(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn risk_context_default_represents_absent_context() {
        let context = RiskContext::default();

        assert!(context.customer_segment().is_none());
        assert!(context.merchant_vertical().is_none());
        assert_eq!(context.email_status(), VerificationStatus::NotProvided);
        assert!(context.device().is_none());
        assert!(context.network().is_none());
        assert!(context.velocity().is_none());
    }

    #[test]
    fn risk_context_accessors_return_configured_values() {
        let device_id = DeviceId::new("device_123").expect("device id should be valid");
        let provider_token = DeviceProviderToken::new("token_123").expect("token should be valid");
        let device = DeviceRiskContext::new()
            .with_device_id(device_id)
            .with_provider_token(provider_token)
            .with_seen_before(true)
            .with_recently_changed(false);
        let network = NetworkRiskContext::new()
            .with_ip_country(CountryCode::new("US").expect("country should be valid"))
            .with_billing_country(CountryCode::new("US").expect("country should be valid"))
            .with_shipping_country(CountryCode::new("CA").expect("country should be valid"))
            .with_proxy_detected(true)
            .with_vpn_detected(false)
            .with_tor_detected(false);
        let velocity = VelocityRiskContext::new()
            .with_attempts_last_10_minutes(2)
            .with_attempts_last_hour(4)
            .with_attempts_last_day(8)
            .with_successful_payments_last_day(3)
            .with_failed_payments_last_day(1)
            .with_chargebacks_last_90_days(0);

        let context = RiskContext::new()
            .with_customer_segment(CustomerSegment::Returning)
            .with_merchant_vertical(MerchantVertical::DigitalGoods)
            .with_account_age_days(30)
            .with_email_status(VerificationStatus::Verified)
            .with_phone_status(VerificationStatus::Pending)
            .with_identity_status(VerificationStatus::NotProvided)
            .with_device(device)
            .with_network(network)
            .with_velocity(velocity);

        assert_eq!(
            context.customer_segment(),
            Some(&CustomerSegment::Returning)
        );
        assert_eq!(
            context.merchant_vertical(),
            Some(&MerchantVertical::DigitalGoods)
        );
        assert_eq!(context.account_age_days(), Some(30));
        assert_eq!(context.email_status(), VerificationStatus::Verified);
        assert_eq!(context.phone_status(), VerificationStatus::Pending);
        assert_eq!(
            context.device().expect("device should exist").seen_before(),
            Some(true)
        );
        assert_eq!(
            context
                .network()
                .expect("network should exist")
                .proxy_detected(),
            Some(true)
        );
        assert_eq!(
            context
                .velocity()
                .expect("velocity should exist")
                .failed_payments_last_day(),
            Some(1)
        );
    }

    #[test]
    fn device_tokens_are_redacted_from_debug() {
        let device_id = DeviceId::new("device_secret").expect("device id should be valid");
        let provider_token =
            DeviceProviderToken::new("provider_secret").expect("token should be valid");

        let debug = format!("{device_id:?} {provider_token:?}");

        assert!(!debug.contains("device_secret"));
        assert!(!debug.contains("provider_secret"));
        assert!(debug.contains("[redacted]"));
    }
}
