# Changelog

All notable changes to PayRail are documented here.

## 0.1.4 - Unreleased

### Added

- Modeled MTN MoMo, M-Pesa, Airtel Money, Flutterwave, and Paystack as reserved built-in provider
  route targets.

## 0.1.3 - Released

### Changed

- Replaced runtime connector registration with static built-in provider dispatch in the PayRail
  facade and router.
- Route configuration now uses `BuiltinProvider` for cheap-copy provider selection.
- Provider connectors expose direct inherent async methods instead of `async_trait` trait-object
  wrappers.
- Updated routing documentation to distinguish route targets from configured executable
  connectors.
- Updated provider extension guidance for first-party, feature-gated payment rails such as MTN
  MoMo, M-Pesa, Circle, Coinbase, Bridge, Binance, and aggregators.
- Reworked router dispatch benchmarks to measure provider resolution without async runtime,
  provider network I/O, or error-path timing.

### Removed

- Removed `PaymentConnector`, `CapturablePaymentConnector`, and `MobileMoneyGateway` trait-based
  extension APIs from the public surface.
- Removed builder `.connector(...)` and `.capturable_connector(...)` dynamic registration APIs.
- Removed the `async-trait` dependency from the workspace.

### Fixed

- Fixed scheduled security workflow action resolution by using the supported install action
  version.
- Fixed fuzz build expectations so fuzz targets compile with `cargo-fuzz` on nightly.
- Fixed crates.io README badge rendering by using badge links that render correctly on crates.io.
- Cleaned README examples so user-facing snippets no longer include hidden doctest setup lines.

## 0.1.0 - Unreleased

### Added

- Initial Rust 2024 workspace.
- Provider-neutral `payrail-core` domain model and connector traits.
- `payrail` facade with feature-gated connector registration and routing.
- Stripe connector for Checkout Sessions, refunds, status mapping, and webhook normalization.
- PayPal connector for OAuth, Orders API creation/status, capture, and webhook parsing.
- Mobile Money helpers for shared operator and phone-number handling.
- Lipila connector for Zambia Mobile Money collections, status checks, callbacks, and HMAC webhook verification.
- Mandatory quality gates for formatting, clippy, tests, docs, coverage, dependency checks, and secret scanning.
