# Provider Extension Contribution Template

Use this template when proposing a new PayRail provider connector, crypto provider, Mobile Money
rail, or aggregator adapter such as Circle, Coinbase, Bridge, Binance, MTN MoMo, M-Pesa, Airtel
Money, Flutterwave, or another Lipila-like provider.

## Architecture Fit

PayRail is intended to be easy to extend:

- `payrail` owns provider-neutral types, errors, idempotency, webhooks, connector traits, and
  first-party provider implementations.
- First-party providers live as internal modules behind feature flags, for example `stripe`,
  `paypal`, and `lipila`.
- The facade crate routes provider-neutral `CreatePaymentRequest` values to configured connectors.
- Capture is modeled as an optional capability through `CapturablePaymentConnector`.
- Mobile Money routing is configurable with `PayRailBuilder::mobile_money_route(country, provider)`,
  so a new aggregator can serve a country without changing core routing.
- Crypto routing is configurable with `PayRailBuilder::crypto_route(provider)` plus more specific
  asset/network routes, so providers such as Circle, Coinbase, Bridge, or Binance can be added
  without changing core routing.

The default Zambia Mobile Money route is Lipila. A custom provider or aggregator can override it:

```rust
use std::sync::Arc;

use payrail::{
    CountryCode, PayRail, PaymentProvider,
};

# async fn build_client(connector: Arc<dyn payrail::PaymentConnector>) -> Result<(), payrail::PaymentError> {
let client = PayRail::builder()
    .connector(connector)
    .mobile_money_route(
        CountryCode::new("ZM")?,
        PaymentProvider::Other("flutterwave".to_owned()),
    )
    .build()?;
# Ok(())
# }
```

Stablecoin USDC Checkout defaults to Stripe for backward compatibility. Other stablecoins such as
USDT require an explicit route, as do general crypto payments. This prevents accidentally sending
unsupported assets such as USDT, BTC, or ETH to a provider that only supports hosted USDC Checkout:

```rust
use std::sync::Arc;

use payrail::{
    CryptoAsset, CryptoNetwork, PayRail, PaymentProvider,
};

# async fn build_crypto_client(connector: Arc<dyn payrail::PaymentConnector>) -> Result<(), payrail::PaymentError> {
let client = PayRail::builder()
    .connector(connector)
    .crypto_route(PaymentProvider::Coinbase)
    .crypto_asset_route(CryptoAsset::Usdt, PaymentProvider::Binance)
    .crypto_asset_network_route(
        CryptoAsset::Usdc,
        CryptoNetwork::Base,
        PaymentProvider::Circle,
    )
    .build()?;
# Ok(())
# }
```

PayRail reserves dedicated provider variants for planned first-party crypto adapters such as
`PaymentProvider::Circle`, `PaymentProvider::Coinbase`, `PaymentProvider::Bridge`, and
`PaymentProvider::Binance`. External or experimental connectors should use
`PaymentProvider::Other(name)`.

## Adding Stablecoin Support

Stablecoin support is intentionally extensible. A new provider connector usually does not need a
PayRail core change to support a new stablecoin:

```rust
use payrail::{
    CryptoAsset, PaymentMethod, PaymentProvider, StablecoinAsset,
};

# fn example() -> Result<(), payrail::PaymentError> {
let client = payrail::PayRail::builder()
    .crypto_asset_route(
        CryptoAsset::Other("eurc".to_owned()),
        PaymentProvider::Other("stablecoin-provider".to_owned()),
    )
    .build()?;

let method = PaymentMethod::stablecoin(StablecoinAsset::Other("eurc".to_owned()));
# let _ = (client, method);
# Ok(())
# }
```

Use `StablecoinAsset::Other(symbol)` and `CryptoAsset::Other(symbol)` for provider-specific,
experimental, regional, or newly launched stablecoins. Prefer lowercase canonical symbols in
examples and tests unless the provider requires a different wire format; provider connectors should
normalize to the provider's expected representation at the boundary.

Add a first-class enum variant only when the asset is broadly supported or expected to be routed by
many PayRail users. A first-class stablecoin contribution should update:

- `StablecoinAsset` in PayRail core types.
- `CryptoAsset` in PayRail core types when the asset can also be used with network-specific crypto
  routing.
- The `From<&StablecoinAsset> for CryptoAsset` mapping.
- Convenience constructors only when they improve ergonomics, for example
  `PaymentMethod::stablecoin_usdt()` or `PaymentMethod::usdt_on(network)`.
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

## Module Layout

First-party providers should be added as feature-gated modules inside the `payrail` crate. External
or experimental providers may live in separate crates that depend on `payrail`:

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
- `payout.rs` for providers that separate payment acceptance from settlement/payout.
- `callback.rs` for callback payload normalization.

Keep `mod.rs` minimal: module declarations and public re-exports only.

## Required Connector Design

Implement `PaymentConnector` for every provider:

```rust
#[async_trait::async_trait]
impl payrail::PaymentConnector for ProviderConnector {
    fn provider(&self) -> payrail::PaymentProvider {
        payrail::PaymentProvider::Other("provider-name".to_owned())
    }

    async fn create_payment(
        &self,
        request: payrail::CreatePaymentRequest,
    ) -> Result<payrail::PaymentSession, payrail::PaymentError> {
        // Validate supported method/country/currency.
        // Build provider request.
        // Send request with timeout and redacted tracing.
        // Return normalized PaymentSession.
    }

    async fn get_payment_status(
        &self,
        provider_reference: &payrail::ProviderReference,
    ) -> Result<payrail::PaymentStatusResponse, payrail::PaymentError> {
        // Return normalized status.
    }

    async fn refund_payment(
        &self,
        request: payrail::RefundRequest,
    ) -> Result<payrail::RefundResponse, payrail::PaymentError> {
        // Implement or return UnsupportedOperation.
    }

    async fn parse_webhook(
        &self,
        request: payrail::WebhookRequest<'_>,
    ) -> Result<payrail::PaymentEvent, payrail::PaymentError> {
        // Verify signature before parsing whenever provider supports signatures.
    }
}
```

If the provider supports capture, also implement `CapturablePaymentConnector`.

## Configuration Requirements

- Store secrets as `secrecy::SecretString`.
- Do not derive `Serialize` for secret-bearing config types.
- Read secrets from environment variables in examples; never hard-code credentials.
- Provide sandbox and production constructors when the provider has separate environments.
- Reject production/live mode in debug/test builds unless `PAYRAIL_ALLOW_LIVE_TESTS=true`.
- Use `reqwest::Client::builder().timeout(...)`.
- Reject zero request timeouts.

## Payment and Routing Rules

- Validate country and currency before sending provider requests.
- For crypto providers, validate requested asset, network, fiat currency, settlement currency, and
  hosted-checkout constraints before sending provider requests.
- Return `UnsupportedPaymentMethod`, `UnsupportedCountry`, `UnsupportedCurrency`, or
  `UnsupportedPaymentRoute` before making an HTTP call.
- Do not expose raw provider responses in public return types.
- Do not include card numbers, wallet private keys, seed phrases, deposit private keys, raw wallet
  credentials, or raw sensitive account data in any API.
- For Mobile Money, route countries explicitly from the facade:

```rust
let client = PayRail::builder()
    .connector(Arc::new(provider_connector))
    .mobile_money_route(CountryCode::new("KE")?, PaymentProvider::Other("mpesa".to_owned()))
    .mobile_money_route(CountryCode::new("UG")?, PaymentProvider::Other("airtel".to_owned()))
    .build()?;
```

An aggregator that supports several countries should document every route it expects applications
to register.

For crypto providers, route explicitly from the facade:

```rust
let client = PayRail::builder()
    .connector(Arc::new(coinbase_connector))
    .connector(Arc::new(circle_connector))
    .crypto_route(PaymentProvider::Coinbase)
    .crypto_asset_route(CryptoAsset::Usdt, PaymentProvider::Binance)
    .crypto_network_route(CryptoNetwork::Solana, PaymentProvider::Bridge)
    .crypto_asset_network_route(
        CryptoAsset::Usdc,
        CryptoNetwork::Base,
        PaymentProvider::Circle,
    )
    .build()?;
```

Crypto route precedence is:

1. Asset + network route.
2. Asset route.
3. Network route.
4. Default crypto route.

Use `PaymentMethod::stablecoin(asset)` for stablecoin checkout when the network does not matter,
including `PaymentMethod::stablecoin_usdt()` for USDT. Use `PaymentMethod::crypto(asset)` for
general crypto when the network does not matter. Use `PaymentMethod::crypto_on(asset, network)`,
`PaymentMethod::usdc_on(network)`, or `PaymentMethod::usdt_on(network)` when network choice is part
of the payment contract.

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
