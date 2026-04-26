//! Lipila connector for PayRail.

#![forbid(unsafe_code)]

mod callback;
mod client;
mod collection;
mod config;
mod mapper;
mod models;
mod webhook;

pub use client::LipilaConnector;
pub use config::{LipilaConfig, LipilaEnvironment};
