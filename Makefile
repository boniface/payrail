SHELL := /bin/sh

CARGO ?= cargo
RUSTDOCFLAGS ?= -D warnings
LLVM_COV_ARGS ?= --workspace --all-features --fail-under-lines 90
FUZZ_RUNS ?= 100

.PHONY: help
help:
	@printf '%s\n' \
		'PayRail developer targets:' \
		'' \
		'  make fmt           Format all Rust code' \
		'  make fmt-check     Check rustfmt without writing changes' \
		'  make check         cargo check for the full workspace' \
		'  make cjeck         Alias for make check' \
		'  make lint          clippy with -D warnings' \
		'  make test          Run workspace tests with all features' \
		'  make examples      Compile example targets' \
		'  make doc           Build docs with warnings denied' \
		'  make coverage      Run llvm-cov with the release coverage gate' \
		'  make deny          Run cargo-deny' \
		'  make audit         Run cargo-audit' \
		'  make secrets       Run gitleaks secret scan' \
		'  make machete       Run unused dependency check' \
		'  make fuzz-check    Check fuzz harness compilation' \
		'  make fuzz-smoke    Run bounded fuzz smoke tests' \
		'  make package       Verify package contents for the public payrail crate' \
		'  make publish-dry-run Dry-run publish the public payrail crate' \
		'  make ci            Main local CI gate' \
		'  make security      Security-focused checks' \
		'  make all           CI + security + package'

.PHONY: fmt
fmt:
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check:
	$(CARGO) fmt --all -- --check

.PHONY: check cjeck
check:
	$(CARGO) check --workspace --all-features

cjeck: check

.PHONY: lint
lint:
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings

.PHONY: test
test:
	$(CARGO) test --workspace --all-features

.PHONY: examples
examples:
	$(CARGO) test --examples --all-features -p payrail

.PHONY: doc
doc:
	RUSTDOCFLAGS="$(RUSTDOCFLAGS)" $(CARGO) doc --workspace --all-features --no-deps

.PHONY: coverage
coverage:
	$(CARGO) llvm-cov $(LLVM_COV_ARGS)

.PHONY: deny
deny:
	$(CARGO) deny check

.PHONY: audit
audit:
	$(CARGO) audit

.PHONY: secrets
secrets:
	gitleaks detect --source . --config gitleaks.toml --no-git

.PHONY: machete
machete:
	$(CARGO) machete

.PHONY: fuzz-check
fuzz-check:
	$(CARGO) check --manifest-path fuzz/Cargo.toml

.PHONY: fuzz-smoke
fuzz-smoke:
	$(CARGO) fuzz run core_validation -- -runs=$(FUZZ_RUNS)
	$(CARGO) fuzz run stripe_webhook -- -runs=$(FUZZ_RUNS)
	$(CARGO) fuzz run lipila_webhook -- -runs=$(FUZZ_RUNS)

.PHONY: package
package:
	$(CARGO) package --allow-dirty -p payrail

.PHONY: publish-dry-run
publish-dry-run:
	$(CARGO) publish --dry-run --allow-dirty -p payrail

.PHONY: ci
ci: fmt-check check lint test examples doc coverage

.PHONY: security
security: deny audit secrets machete fuzz-check

.PHONY: all
all: ci security package
