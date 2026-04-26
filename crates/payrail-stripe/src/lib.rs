//! Stripe connector for PayRail.

#![forbid(unsafe_code)]

mod client;
mod config;
mod mapper;
mod models;
mod webhook;

pub use client::StripeConnector;
pub use config::StripeConfig;
