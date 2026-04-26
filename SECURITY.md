# Security Policy

PayRail handles payment orchestration code and treats security issues as high priority.

## Supported Versions

Only the latest unreleased `main` branch is currently supported. Versioned support windows will be documented before the first stable release.

## Reporting a Vulnerability

Do not open public issues for vulnerabilities.

Report security concerns by emailing the repository owner or using GitHub private vulnerability reporting when enabled for `github.com/boniface/payrail`.

Include:

- Affected crate and version or commit.
- Reproduction steps.
- Expected and actual behavior.
- Whether secrets, payment data, customer data, or webhook verification are involved.

## Security Requirements

- Never commit secrets, API keys, OAuth tokens, webhook secrets, or live credentials.
- Use `secrecy::SecretString` for secret-bearing configuration.
- Do not serialize secret-bearing config structs.
- Do not log authorization headers, API keys, OAuth tokens, webhook secrets, raw provider responses, raw webhook bodies, phone numbers, emails, or other PII.
- Verify webhooks using the raw request body.
- Use constant-time comparisons for HMAC/signature checks.
- Reject stale replayable webhook timestamps where the provider supplies timestamps.
- Keep sandbox and live tests gated by explicit environment variables.

## Verification Standard

PayRail uses OWASP ASVS Level 2 as its default review bar and Level 3 rigor for webhooks, secrets, cryptographic comparisons, idempotency, and logging.
