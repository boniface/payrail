# PayRail Security Review

This document records the baseline security posture for the `0.1.0` release candidate.

## Verification Standard

- Baseline: OWASP ASVS Level 2.
- Level 3 rigor applies to webhook verification, secret handling, cryptographic comparisons, idempotency, and logging.

## Reviewed Controls

### Secrets

- Secret-bearing provider configuration uses `secrecy::SecretString`.
- Secret-bearing provider config structs do not derive `Serialize`.
- `.env` and `.env.*` files are ignored by git, except `.env.example`.
- Examples read credentials from environment variables.

### Webhooks

- Stripe webhook verification uses the raw request body and constant-time signature comparison.
- Stripe webhook verification rejects stale timestamps to reduce replay risk.
- PayPal webhook verification uses PayPal's verification endpoint before event parsing.
- Lipila webhook verification uses the raw request body, HMAC-SHA256, constant-time comparison, and stale timestamp rejection.
- Normalized public events do not include raw webhook payloads.

### Provider Responses

- Public payment/session/event types expose normalized fields.
- Raw provider response bodies are not included in default public response types.
- Provider errors use redacted `ProviderErrorDetails`.

### PCI Scope

- PayRail does not collect or transmit raw card numbers.
- Stripe card and stablecoin flows use Stripe-hosted or Stripe-secure payment flows.
- Stripe stablecoin Checkout rejects non-USD line-item currencies and non-USDC stablecoin assets
  before sending provider requests, matching Stripe's current crypto Checkout requirements.
- General crypto payments and non-USDC stablecoins require explicit provider routing before a
  connector receives the request, preventing unsupported assets or networks from falling through
  to Stripe by default.
- PayRail does not custody private keys, seed phrases, or raw wallet credentials.

### Idempotency

- `CreatePaymentRequest` supports idempotency keys.
- Stripe passes idempotency keys through `Idempotency-Key`.
- PayPal passes idempotency keys through `PayPal-Request-Id`.
- Lipila relies on unique merchant references and application-side duplicate prevention.

### Logging and Tracing

- Provider connectors emit only redacted tracing fields: provider name, operation name, payload
  length, idempotency-key presence, and request status where applicable.
- Tracing does not include secret values, authorization headers, raw provider responses, raw webhook
  payloads, phone numbers, or emails.

## Required Follow-Up Before Stable Release

- Add sandbox integration tests behind ignored/gated test flags.

## Coverage Review

The workspace release gate is 90% line coverage. Security-sensitive modules target 95%; modules
below that target require explicit review until additional negative-path tests raise coverage.

| Module | Current line coverage | Review disposition |
| --- | ---: | --- |
| `payrail/src/providers/paypal/auth.rs` | 82.26% | OAuth success and failure are mock-tested; add token-cache expiry and malformed-token tests before stable release. |
| `payrail/src/providers/paypal/client.rs` | 91.53% | Core order, idempotent capture, and webhook verification flows are mock-tested; add provider-error and malformed-response tests for every operation before stable release. |
| `payrail/src/providers/stripe/client.rs` | 84.51% | Checkout, idempotent refund, session-to-payment-intent refund resolution, status, and webhook flows are mock-tested; add missing-field and provider-error tests before stable release. |
| `payrail/src/providers/lipila/client.rs` | 89.60% | Collection, status, and webhook flows are mock-tested; add status-error and malformed-response tests before stable release. |

## Hardening Evidence

- Fuzz targets exist for core validation, Stripe signed webhook parsing, and Lipila signed webhook parsing.
- PayPal webhook verification depends on PayPal's verification endpoint and is covered with mocked HTTP tests rather than a pure local fuzz target.
- Scheduled CI builds and runs each fuzz target for a bounded smoke run.
- Scheduled CI runs mutation testing, semver checks, Miri for `payrail`, and unused dependency checks.
- Release candidates must archive mutation and fuzz results before publishing.
