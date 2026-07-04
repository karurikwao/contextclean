# Redaction

ContextClean redacts secret-like values by default. Redaction is defensive and irreversible in output.

## Current Patterns

Phase 1 redacts:

- private key blocks
- common assignments such as `API_KEY=...`, `TOKEN=...`, `PASSWORD=...`, `DATABASE_URL=...`
- bearer tokens
- JWT-like values

## Guarantees

- Redaction runs before rendering.
- Redacted values are replaced with `[REDACTED_SECRET]`.
- Disabling redaction requires `--no-redact-secrets`.
- The CLI warns when redaction is disabled unless `--quiet` is used.

## Limitations

No secret detector is perfect. ContextClean should be treated as a guardrail, not as a substitute for security review.

Phase 1 assignment redaction is intentionally conservative and works best on single-token values such as `API_KEY=value`, `TOKEN=value`, and `DATABASE_URL=value`. Quoted secrets containing whitespace may not be fully detected yet.
