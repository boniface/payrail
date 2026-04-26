# Contributing

PayRail is a payment library, so changes must be conservative, tested, and security-aware.

## Development Requirements

- Use the current stable Rust toolchain documented in `Cargo.toml`.
- Keep all crates on Rust 2024 edition.
- Do not use `unsafe`; the workspace forbids unsafe code.
- Do not log secrets, authorization headers, customer phone numbers, customer emails, raw provider responses, or raw webhook bodies.
- Keep `mod.rs` files minimal when introduced: module declarations and `pub use` re-exports only.

## Required Checks

Run these before submitting changes:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --examples --all-features -p payrail
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

Release-candidate work must also pass:

```bash
cargo llvm-cov --workspace --all-features --fail-under-lines 90
cargo deny check
cargo audit
```

## Testing Expectations

- Unit tests are mandatory for all new behavior.
- Mock-backed integration tests are mandatory for every provider HTTP operation.
- Webhook code must include positive and negative signature tests.
- Redaction tests are required for any diagnostic path that could contain provider or customer data.
- Live sandbox tests must be ignored by default and gated by explicit environment variables.

## Pull Request Scope

Keep changes narrowly scoped. Avoid unrelated refactors, broad formatting churn, or provider-specific types leaking into `payrail-core`.
