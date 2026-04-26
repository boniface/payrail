//! PayRail facade crate.

#![forbid(unsafe_code)]

mod builder;
mod client;
mod router;

pub use builder::PayRailBuilder;
pub use client::PayRailClient;
pub use payrail_core::*;
pub use router::PaymentRouter;

#[cfg(feature = "lipila")]
pub use payrail_lipila::LipilaConfig;
#[cfg(feature = "mobile-money")]
pub use payrail_mobile_money::*;
#[cfg(feature = "paypal")]
pub use payrail_paypal::PayPalConfig;
#[cfg(feature = "stripe")]
pub use payrail_stripe::StripeConfig;

/// Facade entry point.
#[derive(Debug, Clone, Copy)]
pub struct PayRail;

impl PayRail {
    /// Starts a PayRail client builder.
    #[inline]
    pub fn builder() -> PayRailBuilder {
        PayRailBuilder::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payrail_returns_builder() {
        let builder = PayRail::builder();

        assert!(format!("{builder:?}").contains("PayRailBuilder"));
    }
}
