# payrail-core

Provider-neutral domain types, connector traits, errors, idempotency primitives, and webhook abstractions for PayRail.

This crate must not depend on provider SDKs or leak provider-specific models into public APIs.

## Security

Raw provider responses and raw webhook payloads are intentionally absent from default public response types. Use normalized fields and redacted diagnostics.
