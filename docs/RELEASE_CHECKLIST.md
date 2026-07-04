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

## Release Notes

Release notes must include:

- user-facing changes
- known limitations
- verification commands
- upgrade notes if applicable
