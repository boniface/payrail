# PayRail Release Checklist

Use this checklist before publishing a PayRail release.

## Manual GitHub Release

Releases are intentionally manual from GitHub Actions. Use the `Release` workflow,
provide the exact semver version without a leading `v`, and select one of:

- `dry-run`: runs the full validation gate and packaging checks without publishing.
- `publish`: runs the same validation gate, publishes the single public `payrail` crate to
  crates.io, then creates a GitHub release tagged as `v<version>`.

Publish mode must be run from `main` and fails if the `v<version>` tag already exists.
The repository or `crates-io` environment must define `CARGO_REGISTRY_TOKEN` with
permission to publish the `payrail` crate. The `crates-io` environment should require
manual approval so a publish cannot happen from a single accidental click.

Before running `publish`, update and commit:

- `[workspace.package] version` in `Cargo.toml`.
- Changelog or release notes, when applicable.

## Accidental Internal Crate Releases

The old internal package names are no longer workspace members or repository packages. If an
internal crate version was accidentally published, use the `Yank Internal Crates` workflow with the
affected version, usually `0.1.0`.

This yanks:

- `payrail-core`
- `payrail-stripe`
- `payrail-paypal`
- `payrail-mobile-money`
- `payrail-lipila`

Yanking does not delete a crate from crates.io, but it prevents new dependency resolution from
selecting that version. Existing lockfiles can still build it.

## Required Checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --examples --all-features -p payrail
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo llvm-cov --workspace --all-features --fail-under-lines 90
cargo deny check
cargo audit
gitleaks detect --source . --config gitleaks.toml
```

The GitHub `Release` workflow runs these checks before publishing. Running them locally
first shortens the feedback cycle and avoids consuming a manual release approval on
issues that can be caught before the workflow starts.

## Publish Command

PayRail publishes one public crate. First-party providers are internal modules behind Cargo
features, not separately published crates.

```bash
cargo package --allow-dirty -p payrail
cargo publish --dry-run -p payrail
cargo publish -p payrail
```

## Manual Review

- Confirm crate names are reserved or available.
- Confirm crate metadata, README files, docs.rs links, license expression, repository URL, keywords, and categories.
- Confirm no public API exposes raw provider responses or raw webhook payloads.
- Confirm examples compile and do not contain real credentials.
- Confirm sandbox tests are ignored by default and refuse live credentials unless explicitly enabled.
- Confirm security review findings are resolved or documented.
- Confirm `docs/security-review.md` has current mutation, fuzzing, Miri, semver, and unused-dependency notes.
