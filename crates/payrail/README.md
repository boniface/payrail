# payrail

Provider-neutral Rust payments crate for PayRail.

Use this crate from applications. It exposes provider-neutral types and registers first-party
provider connectors behind feature flags.

## Features

- `stripe`: enables Stripe card and stablecoin Checkout routing.
- `paypal`: enables PayPal Orders routing and capture.
- `lipila`: enables Lipila Zambia Mobile Money routing.
- `mobile-money`: enables shared Mobile Money helpers.
- `all-providers`: enables every provider.
- `rustls`: default TLS backend.
- `native-tls`: optional native TLS backend.

## Security

PayRail does not collect raw card numbers. Do not log secrets, raw webhook bodies, raw provider responses, phone numbers, or emails.
