//! Shared Mobile Money abstractions for PayRail.

#![forbid(unsafe_code)]

mod gateway;
mod operator;
mod phone;

pub use gateway::MobileMoneyGateway;
pub use operator::ZambiaOperator;
pub use payrail_core::{MobileMoneyOperator, MobileMoneyPaymentMethod, PhoneNumber};
pub use phone::normalize_zambia_phone;
