# Changelog

All notable changes to PayRail are documented here.

## 0.2.0 - Released

### Added

- Added the `fraud` feature with provider-neutral fraud types, including risk scores, risk
  decisions, risk levels, fraud reasons, risk context, fraud provider metadata, fraud events, and
  risk-aware payment sessions.
- Added `FraudPolicy` as a top-level re-export when `fraud` is enabled, with observe-only and
  enforce modes for local deterministic risk checks.
- Added risk-aware payment creation through `create_payment_with_risk`, returning a successful
  risk-aware session with `payment() == None` when policy rejects before provider I/O.
- Added `parse_fraud_webhook` support for normalized fraud and dispute visibility.
- Added Stripe fraud/dispute webhook normalization for supported dispute events.
- Added a mocked fraud provider harness and mocked tests for fraud provider behavior without
  selecting a real external fraud vendor.
- Added the `telemetry` feature with structured `tracing` spans and low-cardinality events across
  payment, router, provider, webhook, and fraud policy paths.
- Added the optional `otel` feature with OpenTelemetry API metric helpers for payment requests,
  provider requests, provider latency, webhooks, fraud assessments, and fraud policy blocks.
- Added an `opentelemetry_metrics` example that shows application-owned subscriber, SDK, and OTLP
  exporter setup without making exporters normal library dependencies.

### Changed

- Payment, provider, webhook, and fraud diagnostics now use stable `payrail.*` field names and
  normalized operation names.
- OpenTelemetry SDK/exporter setup remains application-owned; the library only exposes optional API
  helpers and never installs a global subscriber.
- Release automation now publishes the reviewed `CHANGELOG.md` section as the GitHub release body.

### Fixed

- Strengthened telemetry tests so the workspace coverage gate stays above 90% on each staged
  telemetry PR.
- Kept telemetry attributes and metric labels low-cardinality by recording booleans, normalized
  names, status categories, payload length, and error kinds instead of raw identifiers.

### Security

- Telemetry and fraud diagnostics explicitly avoid logging secrets, authorization headers, webhook
  secrets, idempotency keys, customer email, phone numbers, raw webhook payloads, raw provider
  responses, provider references, payment IDs, device tokens, raw risk context, card data, and bank
  details.
- Fraud policy rejection avoids exposing provider-specific fraud details to end users while keeping
  normalized risk decisions available to application/admin telemetry.

## 0.1.7 - Released

### Fixed

- Updated vulnerable transitive dependencies so dependency audit and deny checks pass:
  `crossbeam-epoch` to `0.9.20`, `quinn-proto` to `0.11.16`, and `anyhow` to `1.0.104`.
- Removed stale `cargo-deny` duplicate skip entries that no longer matched the lockfile.

## 0.1.6 - Released

### Added

- Added `CheckoutUiMode::Custom` for Stripe custom Checkout Sessions.
- Added provider-neutral payment object metadata through `payment_metadata`.
- Forward Stripe Checkout Session metadata, PaymentIntent metadata, and customer email when
  creating Checkout Sessions.

### Changed

- `CheckoutUiMode::Elements` remains source-compatible and now maps to Stripe `ui_mode=custom`.

## 0.1.5 - Released

### Added

- Added Stripe embedded Checkout Sessions support through `CheckoutUiMode::Elements` and
  `NextAction::EmbeddedCheckout`.

## 0.1.4 - Released

### Added

- Modeled MTN MoMo, M-Pesa, Airtel Money, Orange Money, Flutterwave, and Paystack as reserved
  built-in provider route targets.

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
