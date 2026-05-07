# Provider Extension Contribution Template

Use this template when proposing a new PayRail provider connector, crypto provider, Mobile Money
rail, or aggregator adapter such as Circle, Coinbase, Bridge, Binance, MTN MoMo, M-Pesa, Airtel
Money, Flutterwave, Paystack, or another Lipila-like provider.

## Current Extension Model

PayRail uses static first-party dispatch:

- `payrail` owns provider-neutral types, errors, idempotency, webhooks, route configuration, and
  first-party provider implementations.
- First-party providers live as internal modules behind feature flags, for example `stripe`,
  `paypal`, and `lipila`.
- The facade routes provider-neutral requests to concrete connector fields. There is no runtime
  trait-object connector registry.
- Route configuration uses `BuiltinProvider`, which is cheap to copy and suitable for hot paths.
- `PaymentProvider::Other(...)` is metadata for normalized provider references and events. It is not
  accepted by route configuration APIs.

Route configuration and connector availability are separate. A route can resolve to a modeled
provider such as `BuiltinProvider::Coinbase` or `BuiltinProvider::MtnMomo`, but payment execution
returns `ConnectorNotConfigured` until a matching first-party connector exists and is configured.

## Current Provider Matrix

| Provider | Status | Route target | Connector feature |
| --- | --- | --- | --- |
| Stripe | Implemented | `BuiltinProvider::Stripe` | `stripe` |
| PayPal | Implemented | `BuiltinProvider::PayPal` | `paypal` |
| Lipila | Implemented | `BuiltinProvider::Lipila` | `lipila` |
| Circle | Reserved crypto target | `BuiltinProvider::Circle` | Not implemented |
| Coinbase | Reserved crypto target | `BuiltinProvider::Coinbase` | Not implemented |
| Bridge | Reserved crypto target | `BuiltinProvider::Bridge` | Not implemented |
| Binance | Reserved crypto target | `BuiltinProvider::Binance` | Not implemented |
| MTN MoMo | Reserved Mobile Money target | `BuiltinProvider::MtnMomo` | Not implemented |
| M-Pesa | Reserved Mobile Money target | `BuiltinProvider::Mpesa` | Not implemented |
| Airtel Money | Reserved Mobile Money target | `BuiltinProvider::AirtelMoney` | Not implemented |
| Flutterwave | Reserved aggregator target | `BuiltinProvider::Flutterwave` | Not implemented |
| Paystack | Reserved aggregator target | `BuiltinProvider::Paystack` | Not implemented |

## Routing Rules

Mobile Money routing is country-based. Lipila is the default Zambia route:

```rust
use payrail::{BuiltinProvider, CountryCode, PayRail};

let client = PayRail::builder()
    .mobile_money_route(CountryCode::new("ZM")?, BuiltinProvider::Lipila)
    .build()?;
```

Do not document or configure a country route unless the selected provider supports that country.
For example, MTN MoMo Ghana already has a reserved route target, but it still needs a first-party
connector before applications can execute payments through it:

```rust,ignore
let client = PayRail::builder()
    .mtn_momo(mtn_config)?
    .mobile_money_route(CountryCode::new("GH")?, BuiltinProvider::MtnMomo)
    .build()?;
```

Crypto routing precedence is:

1. Asset + network route.
2. Asset route.
3. Network route.
4. Default crypto route.

Reserved crypto route targets can be used to validate route behavior before connector work, but
successful payment execution requires the connector:

```rust
use payrail::{BuiltinProvider, CryptoAsset, CryptoNetwork, PayRail, PaymentMethod};

let client = PayRail::builder()
    .crypto_route(BuiltinProvider::Coinbase)
    .crypto_asset_route(CryptoAsset::Usdt, BuiltinProvider::Binance)
    .crypto_network_route(CryptoNetwork::Solana, BuiltinProvider::Bridge)
    .crypto_asset_network_route(
        CryptoAsset::Usdc,
        CryptoNetwork::Base,
        BuiltinProvider::Circle,
    )
    .build()?;

let method = PaymentMethod::usdc_on(CryptoNetwork::Base);
```

## Adding A First-Party Provider

First-party providers should be added as feature-gated modules inside the `payrail` crate:

```text
crates/payrail/src/providers/provider_name/
  mod.rs
  config.rs
  client.rs
  mapper.rs
  models.rs
  webhook.rs
crates/payrail/tests/provider_name_mock_backend.rs
```

Optional modules:

- `auth.rs` for OAuth or token caching.
- `quote.rs` for crypto quotes, fees, or exchange-rate lock payloads.
- `collection.rs` for Mobile Money collection request construction.
- `orders.rs` for checkout/order request construction.
- `payout.rs` for providers that separate payment acceptance from settlement or payout.
- `callback.rs` for callback payload normalization.

Keep `mod.rs` minimal: module declarations and public re-exports only.

Required code changes:

- Add or reuse a `BuiltinProvider` variant in `crates/payrail/src/core/provider.rs`.
- Add the matching `PaymentProvider` variant when public normalized responses need it.
- Add a Cargo feature in `crates/payrail/Cargo.toml`.
- Add a provider module under `crates/payrail/src/providers/` and re-export its public config and
  connector.
- Add a concrete optional connector field to `PaymentRouter`.
- Add builder registration, for example `PayRailBuilder::mtn_momo(config)`.
- Add static dispatch arms for create, status, refund, capture when supported, and webhooks.
- Add route tests proving unsupported routes fail before provider I/O.
- Add mocked backend integration tests for every provider HTTP operation.

Reserved Mobile Money and aggregator targets already modeled in core:

- `BuiltinProvider::MtnMomo` / `PaymentProvider::MtnMomo`
- `BuiltinProvider::Mpesa` / `PaymentProvider::Mpesa`
- `BuiltinProvider::AirtelMoney` / `PaymentProvider::AirtelMoney`
- `BuiltinProvider::Flutterwave` / `PaymentProvider::Flutterwave`
- `BuiltinProvider::Paystack` / `PaymentProvider::Paystack`

When implementing one of these providers, reuse the existing variants instead of adding another
provider ID.

## Configuration Requirements

- Store secrets as `secrecy::SecretString`.
- Do not derive `Serialize` for secret-bearing config types.
- Read secrets from environment variables in examples; never hard-code credentials.
- Provide sandbox and production constructors when the provider has separate environments.
- Reject production/live mode in debug/test builds unless `PAYRAIL_ALLOW_LIVE_TESTS=true`.
- Use `reqwest::Client::builder().timeout(...)`.
- Reject zero request timeouts.

## Payment and Routing Requirements

- Validate payment method, country, currency, asset, and network before sending provider requests.
- Return `UnsupportedPaymentMethod`, `UnsupportedCountry`, `UnsupportedCurrency`, or
  `UnsupportedPaymentRoute` before making an HTTP call.
- Do not expose raw provider responses in public return types.
- Do not include card numbers, wallet private keys, seed phrases, deposit private keys, raw wallet
  credentials, or raw sensitive account data in any API.
- Use `PaymentMethod::stablecoin(asset)` when the network does not matter.
- Use `PaymentMethod::crypto(asset)` when the network does not matter.
- Use `PaymentMethod::crypto_on(asset, network)`, `PaymentMethod::usdc_on(network)`, or
  `PaymentMethod::usdt_on(network)` when network choice is part of the payment contract.

## Adding Stablecoin Support

Use `StablecoinAsset::Other(symbol)` and `CryptoAsset::Other(symbol)` for provider-specific,
experimental, regional, or newly launched stablecoins. Prefer lowercase canonical symbols in
examples and tests unless the provider requires a different wire format.

Add a first-class enum variant only when the asset is broadly supported or expected to be routed by
many PayRail users. A first-class stablecoin contribution should update:

- `StablecoinAsset`.
- `CryptoAsset` when the asset can also be used with network-specific crypto routing.
- The `From<&StablecoinAsset> for CryptoAsset` mapping.
- Convenience constructors only when they improve ergonomics.
- Router tests proving unsupported stablecoins do not fall through to Stripe.
- Provider connector tests proving supported and unsupported assets are handled explicitly.
- README and this extension template.

Security rule: never route a new stablecoin to a provider by default unless that provider explicitly
documents support for the exact asset and network. Non-USDC stablecoins should require explicit
asset or asset+network routing unless the library later adds a provider-backed default with tests.

## Proposal Summary

- Provider or aggregator name:
- Provider category:
  - Crypto checkout/wallet provider
  - Stablecoin orchestration provider
  - Direct Mobile Money operator
  - Mobile Money aggregator
  - Card/checkout provider
  - Bank transfer provider
  - Other
- Countries supported:
- Currencies supported:
- Crypto assets supported:
- Crypto networks supported:
- Payment methods supported:
- Operations supported:
  - Create payment/collection
  - Status lookup
  - Capture
  - Refund
  - Webhook parsing
  - Webhook signature verification
- Provider docs URL:
- Sandbox availability:
- Production API availability:

## Idempotency

- `CreatePaymentRequest` may include an idempotency key; pass it to the provider when supported.
- `RefundRequest` and `CaptureRequest` always include idempotency keys; pass them to the provider
  for retry-safe write operations.
- If the provider lacks idempotency headers, document the duplicate-prevention strategy.
- For crypto providers, document whether the provider treats quotes, checkout sessions, deposits,
  transfers, refunds, and payouts as separately idempotent operations.

## Crypto-Specific Security Requirements

- PayRail must not custody private keys, seed phrases, or raw wallet credentials.
- Prefer hosted checkout, provider-generated deposit addresses, or provider-secure wallet flows.
- Validate chain/network before presenting an address or payment instruction.
- Document finality assumptions, confirmation counts, expiration windows, and underpayment or
  overpayment handling.
- Normalize provider events such as payment pending, confirmed, expired, failed, refunded, and
  chargeback/dispute-equivalent states where the provider supports them.
- Redact wallet addresses in logs unless they are explicitly safe for the application context.
- Do not treat on-chain transaction hashes as secrets, but avoid logging them by default because
  they can reveal customer activity.

## Webhooks

- Verify signatures before parsing payloads whenever provider signatures are available.
- Use the raw request body for signature verification.
- Use constant-time comparison for HMAC/signature checks where applicable.
- Reject stale timestamps when the provider includes signed timestamps.
- Return a stable `WebhookEventId` when the provider supplies one.
- Never log raw webhook bodies.

## Error Handling

- Use `PaymentError` for public errors.
- Use `ProviderErrorDetails` for provider HTTP failures.
- Keep provider messages redacted and safe for application logs.
- Preserve provider request IDs when available and safe.
- Do not use `.unwrap()` in production code paths.

## Tracing and Logging

Only emit redacted tracing fields:

- provider name
- operation name
- HTTP status
- payload length
- idempotency-key presence

Do not log:

- API keys
- OAuth tokens
- authorization headers
- raw provider responses
- raw webhook payloads
- cardholder data
- phone numbers
- email addresses
- national IDs or other PII

## Mandatory Tests

Unit tests:

- Config validation.
- Request body construction.
- Status and event mapping.
- Unsupported method/country/currency behavior.
- Unsupported crypto asset/network behavior.
- Webhook signature verification.
- Idempotency header behavior.
- Secret redaction behavior.
- Router resolution and connector execution behavior.

Mocked backend integration tests under a provider-specific file such as
`crates/payrail/tests/provider_name_mock_backend.rs`:

- Successful create payment/collection.
- Status lookup.
- Capture if supported.
- Refund if supported, or explicit `UnsupportedOperation`.
- Webhook verification and normalization.
- Provider 4xx/5xx responses.
- Malformed JSON.
- Missing required fields.
- Idempotency headers on write operations.
- Crypto route selection for default, asset, network, and asset+network routes when applicable.

Fuzz tests when parsing signed or provider-controlled payloads:

- Webhook payload parser.
- Signature header parser.
- Status/event mapping when input space is broad.

## Required Local Checks

Run these before opening a PR:

```bash
make fmt-check
make check
make lint
make test
make examples
make doc
make coverage
make security
make package
```

For a new fuzz target, also run:

```bash
make fuzz-smoke
```

## Release Note Labels

Every PR should include at least one release-note label so GitHub can generate useful categorized
release notes:

- Use `breaking-change` or `semver-major` for breaking API or behavior changes.
- Use `security` for security posture, vulnerability, webhook, secret-handling, or compliance
  changes.
- Use provider labels such as `provider`, `stripe`, `paypal`, `lipila`, `mobile-money`, or `crypto`
  for payment-rail changes.
- Use `feature`, `enhancement`, `bug`, `fix`, `documentation`, `docs`, `ci`, `release`, or
  `dependencies` for routine changes.
- Use `ignore-for-release` only when a PR should be excluded from generated release notes.

## PR Checklist

- [ ] New provider module is isolated from unrelated providers.
- [ ] `mod.rs` only declares modules and re-exports public API.
- [ ] Config secrets use `SecretString`.
- [ ] No secret-bearing config derives `Serialize`.
- [ ] HTTP client has a configurable timeout.
- [ ] Payment method, country, and currency are validated before provider calls.
- [ ] Crypto asset and network are validated before provider calls.
- [ ] Write operations pass idempotency keys or document provider limitations.
- [ ] Webhook signatures are verified before parsing.
- [ ] Public return types are normalized and do not expose raw provider payloads.
- [ ] Mocked backend integration tests cover all provider operations.
- [ ] Coverage remains above the workspace release gate.
- [ ] `make security` passes.
- [ ] README documents environment variables, sandbox behavior, limitations, and examples.
- [ ] PR has at least one release-note label that matches `.github/release.yml`.

## Documentation Requirements

Every provider PR must document:

- Supported countries and currencies.
- Supported crypto assets and networks.
- Supported payment operations.
- Required environment variables.
- Sandbox setup.
- Webhook setup.
- Idempotency behavior.
- Refund/capture limitations.
- Known provider-specific edge cases.
- Crypto-specific finality, expiration, underpayment, overpayment, and network mismatch behavior.
