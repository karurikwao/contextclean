# Release Checklist

## Pre-Release

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `cargo build --workspace --release`
- [ ] README examples verified
- [ ] `CHANGELOG.md` updated
- [ ] Security and privacy docs reviewed
- [ ] No fixture secrets are real

## Smoke Tests

- [ ] `ctxclean --help`
- [ ] `ctxclean fixtures/dirty_html_small.html`
- [ ] `ctxclean fixtures/repeated_log.txt --format json`
- [ ] `ctxclean fixtures/simple_project --mode standard`
- [ ] `ctxclean fixtures/repeated_log.txt --max-tokens 120`

## Cloudflare Pages

- [ ] Deploy the static site: `npx wrangler pages deploy site --project-name contextclean --branch main`
- [ ] Verify the canonical URL returns HTTP 200: `https://contextclean.pages.dev/`
- [ ] Verify live content includes `Local-first context cleaner for AI agents`
- [ ] Verify security headers are present: `X-Content-Type-Options`, `Referrer-Policy`, `Permissions-Policy`

## Release Notes

Release notes must include:

- user-facing changes
- known limitations
- verification commands
- upgrade notes if applicable
