//! `PayRail` facade crate.

#![forbid(unsafe_code)]

mod builder;
mod client;
mod core;
#[cfg(any(
    feature = "lipila",
    feature = "mobile-money",
    feature = "paypal",
    feature = "stripe"
))]
mod providers;
mod router;

pub use builder::PayRailBuilder;
pub use client::PayRailClient;
pub use core::*;
#[cfg(any(
    feature = "lipila",
    feature = "mobile-money",
    feature = "paypal",
    feature = "stripe"
))]
pub use providers::*;
pub use router::PaymentRouter;

/// Facade entry point.
#[derive(Debug, Clone, Copy)]
pub struct PayRail;

impl PayRail {
    /// Starts a `PayRail` client builder.
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
