# Changelog

All notable changes to PayRail are documented here.

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
