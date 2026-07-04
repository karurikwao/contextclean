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
- common provider tokens such as OpenAI-style `sk-...`, GitHub `ghp_...` and `github_pat_...`, Slack `xox...`, AWS access key ids, and npm tokens

## Guarantees

- Redaction runs before rendering.
- Redacted values are replaced with `[REDACTED_SECRET]`.
- Redaction is enabled by default; `--redact-secrets` is available as an explicit no-op for scripts that want to say the quiet part out loud.
- Disabling redaction requires `--no-redact-secrets`.
- The CLI warns when redaction is disabled unless `--quiet` is used.

## Sensitive Paths

Directory scans skip sensitive paths by default and include a warning in output. Explicit sensitive files and sensitive root directories fail unless `--include-sensitive` is supplied.

Default sensitive path handling covers:

- `.env`, `.env.*`, `.netrc`, `.npmrc`, `.pypirc`, cloud credential files, and local credential stores
- `.ssh`, `.aws`, `.gcloud`, `.azure`, `.kube`, `.docker`, `.terraform`, and similar credential directories
- private keys, certificate-like files, key stores, local databases, and SQLite files

When `--include-sensitive` is used, ContextClean can read sensitive paths that are not excluded by `.gitignore`, global git excludes, or `.ctxcleanignore`. Secret-like values are still redacted unless `--no-redact-secrets` is also supplied.

## Limitations

No secret detector is perfect. ContextClean should be treated as a guardrail, not as a substitute for security review.

Assignment redaction is intentionally conservative and works best on single-token values such as `API_KEY=value`, `TOKEN=value`, `DATABASE_URL=value`, and netrc-style `password value`. Quoted secrets containing whitespace may not be fully detected yet.
