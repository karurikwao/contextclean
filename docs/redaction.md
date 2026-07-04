# Redaction

ContextClean redacts secret-like values by default. Redaction is defensive and irreversible in output.

## Current Patterns

The current implementation redacts:

- private key blocks
- common assignments such as `API_KEY=...`, `TOKEN=...`, `PASSWORD=...`, `DATABASE_URL=...`
- whitespace-delimited secret forms such as `password secretValue`, `token secretValue`, and access-key variants
- sensitive URL query parameters preserved from HTML links, such as `token`, `signature`, `X-Amz-Signature`, and `access_token`
- bearer tokens
- JWT-like values

## Guarantees

- Redaction runs before rendering.
- Redacted values are replaced with `[REDACTED_SECRET]`.
- Disabling redaction requires `--no-redact-secrets`.
- The CLI warns when redaction is disabled unless `--quiet` is used.

## Limitations

No secret detector is perfect. ContextClean should be treated as a guardrail, not as a substitute for security review.

Assignment redaction is intentionally conservative and works best on single-token values such as `API_KEY=value`, `TOKEN=value`, `DATABASE_URL=value`, and netrc-style `password value`. Quoted secrets containing whitespace may not be fully detected yet.
