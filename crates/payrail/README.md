# PayRail

[CI](https://github.com/boniface/payrail/actions/workflows/ci.yml) ·
[Scheduled Security](https://github.com/boniface/payrail/actions/workflows/scheduled-security.yml) ·
[Release](https://github.com/boniface/payrail/actions/workflows/release.yml)

[![Crates.io](https://img.shields.io/crates/v/payrail.svg)](https://crates.io/crates/payrail)
[![Docs.rs](https://docs.rs/payrail/badge.svg)](https://docs.rs/payrail)
[![License](https://img.shields.io/crates/l/payrail.svg)](https://github.com/boniface/payrail#license)
[![Rust Version](https://img.shields.io/badge/rust-1.95%2B-blue.svg)](https://github.com/boniface/payrail/blob/main/Cargo.toml)
[![Rust Edition](https://img.shields.io/badge/edition-2024-blue.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/)
[![Coverage Gate](https://img.shields.io/badge/coverage%20gate-90%25-brightgreen.svg)](https://github.com/boniface/payrail/actions/workflows/ci.yml)
[![Dependencies](https://deps.rs/repo/github/boniface/payrail/status.svg)](https://deps.rs/repo/github/boniface/payrail)

PayRail is a provider-neutral Rust payment library for accepting payments through Stripe, PayPal,
crypto providers, and Mobile Money providers such as Lipila.

The public API exposes PayRail payment concepts instead of provider-specific SDK types. First-party
providers are internal modules behind feature flags so applications only compile the payment rails
they need while depending on one public crate.

## Crate

`payrail` is the only public crate. It contains provider-neutral domain types, idempotency,
webhook abstractions, route configuration, and first-party providers behind Cargo features.

Planned first-party extension points include additional Mobile Money providers, Mobile Money
aggregators, and crypto providers such as Circle, Coinbase, Bridge, and Binance.

## Installation

```toml
[dependencies]
payrail = { version = "0.1", features = ["stripe", "paypal", "lipila"] }
```

The default TLS backend is Rustls. Do not enable all providers unless the application uses them.

Common feature flags:

- `stripe`: Stripe Checkout, status, refunds, and webhooks.
- `paypal`: PayPal Orders, OAuth, capture, and webhooks.
- `lipila`: Lipila Zambia Mobile Money. This also enables `mobile-money`.
- `mobile-money`: shared Mobile Money types and helpers.
- `all-providers`: all first-party providers.
- `rustls`: Rustls TLS backend.
- `native-tls`: native TLS backend.

## Quickstart

```rust
use payrail::{CreatePaymentRequest, Money, PaymentMethod, PayRail};
use payrail::StripeConfig;
use secrecy::SecretString;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PayRail::builder()
        .stripe(StripeConfig::new(SecretString::from(
            std::env::var("STRIPE_SECRET_KEY")?,
        ))?)?
        .build()?;

    let request = CreatePaymentRequest::builder()
        .amount(Money::new_minor(2_500, "USD")?)
        .reference("ORDER-1001")?
        .payment_method(PaymentMethod::card())
        .return_url("https://example.com/success")?
        .cancel_url("https://example.com/cancel")?
        .idempotency_key("ORDER-1001:create")?
        .build()?;

    let session = client.create_payment(request).await?;
    println!("provider reference: {}", session.provider_reference.as_str());

    Ok(())
}
```

## Routing Extensions

Routes use PayRail's built-in provider identifiers and dispatch through concrete connector fields.
There is no trait-object connector registry in the library hot path.

Route configuration and connector availability are separate:

- Implemented connectors today: Stripe, PayPal, and Lipila.
- Reserved crypto route targets: Circle, Coinbase, Bridge, and Binance. These are modeled provider
  IDs, but payments routed to them return `ConnectorNotConfigured` until first-party connectors are
  implemented and configured.
- Unmodeled rails such as MTN MoMo, M-Pesa, Airtel Money, Flutterwave, or Paystack require a
  first-party provider contribution before applications can route traffic to them.
- `PaymentProvider::Other(...)` is metadata for normalized provider references and events. It is not
  a runtime routing extension point.

### Mobile Money Routing

Lipila is the default Zambia Mobile Money route. Applications can override or add country routes
only to modeled built-in providers. Do not route a country to a provider unless that provider
connector supports the country.

```rust
use payrail::{BuiltinProvider, CountryCode, PayRail};

let client = PayRail::builder()
    .mobile_money_route(
        CountryCode::new("ZM")?,
        BuiltinProvider::Lipila,
    )
    .build()?;
```

### Crypto Routing

Stripe-hosted USDC Checkout remains the default stablecoin route for compatibility. Other
stablecoins, including USDT, require explicit routing so unsupported assets or networks do not
accidentally route to a provider that cannot process them.

```rust
use payrail::{
    BuiltinProvider, CryptoAsset, CryptoNetwork, PayRail, PaymentMethod,
};

let client = PayRail::builder()
    .crypto_route(BuiltinProvider::Coinbase)
    .crypto_asset_route(CryptoAsset::Usdt, BuiltinProvider::Coinbase)
    .crypto_asset_network_route(
        CryptoAsset::Usdc,
        CryptoNetwork::Base,
        BuiltinProvider::Circle,
    )
    .build()?;

let method = PaymentMethod::usdc_on(CryptoNetwork::Base);
let usdt = PaymentMethod::stablecoin_usdt();
```

Crypto route precedence is asset + network, then asset, then network, then default crypto route.
Future stablecoins can be supported without changing core by using `StablecoinAsset::Other(symbol)`
and `CryptoAsset::Other(symbol)` with an explicit asset route. Stablecoins that become broadly
supported can be promoted to first-class enum variants with matching routing and provider tests.

The example above configures route selection only. It is useful for validating routing behavior and
for future connector work, but successful payment execution requires the selected provider connector
to exist and be configured.

## Contributing Providers

Provider extensions are implemented inside PayRail as feature-gated first-party modules so the
facade can keep static dispatch and avoid runtime trait-object routing. New providers must add a
`BuiltinProvider` route target when needed, keep secret configuration in `secrecy::SecretString`,
verify webhooks before parsing, and include mocked backend integration tests for provider
operations.

Use the provider contribution template:

- [docs/provider-extension-template.md](https://github.com/boniface/payrail/blob/main/docs/provider-extension-template.md)

## Security Posture

- PayRail never handles raw card numbers; card payments use provider-hosted or provider-secure flows.
- API keys and webhook secrets use `secrecy::SecretString`.
- Webhook verification uses raw request bodies and constant-time comparison where applicable.
- Public response types do not expose raw provider response bodies or raw webhook payloads by default.
- Errors expose only normalized or redacted provider diagnostics.
- Sandbox/live tests are gated by explicit environment variables and never run by default.

## Testing and Quality Gates

Required before release candidates:

```bash
make fmt-check
make check
make lint
make test
make examples
make doc
make coverage
make security
```

The workspace release gate is 90% line coverage. Core and security-sensitive modules target 95%+ line coverage, with branch coverage tracked where tooling permits.

## Provider Limitations

- Stripe stablecoin support is delegated to Stripe-supported Checkout flows.
- General crypto payments use provider-neutral `PaymentMethod::crypto` and explicit route
  registration for modeled providers such as Circle, Coinbase, Bridge, or Binance once their
  first-party connectors exist.
- PayRail does not custody private keys, seed phrases, or raw wallet credentials.
- PayPal refunds are not implemented in v1.
- Lipila v1 exposes Zambia Mobile Money collections only.
- Other Mobile Money providers such as MTN MoMo, M-Pesa, Airtel Money, or aggregators such as
  Flutterwave are future first-party adapters over the shared Mobile Money abstractions.

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license
