# Release Checklist

## Pre-Release

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features --locked`
- [ ] `cargo build --workspace --release --locked`
- [ ] `powershell -ExecutionPolicy Bypass -File .\scripts\benchmarks.ps1`
- [ ] `cargo package -p contextclean-core --locked`
- [ ] `cargo package -p contextclean-cli --locked`
- [ ] `cargo publish -p contextclean-core --dry-run --locked`
- [ ] After `contextclean-core` is available in the registry, run `cargo publish -p contextclean-cli --dry-run --locked`.
- [ ] Publish `contextclean-core` first when publishing to crates.io.
- [ ] Publish `contextclean-cli` only after the core crate is available in the registry.
- [ ] README examples verified
- [ ] `CHANGELOG.md` updated
- [ ] Security and privacy docs reviewed
- [ ] No fixture secrets are real
- [ ] Git remote configured
- [ ] Registry credentials available
- [ ] GitHub Actions green on the release commit

## Smoke Tests

- [ ] `ctxclean --help`
- [ ] `ctxclean gha fixtures/ci_failure_log.txt --format json`
- [ ] `ctxclean repo fixtures/simple_project --format json`
- [ ] PowerShell: `'{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | ctxclean mcp`
- [ ] `ctxrun --help`
- [ ] `ctxclean fixtures/dirty_html_small.html`
- [ ] `ctxclean fixtures/dirty_html_article.html --mode standard --format json`
- [ ] `ctxclean fixtures/repeated_log.txt --format json`
- [ ] `ctxclean fixtures/ci_failure_log.txt --mode standard --format json`
- [ ] `ctxclean fixtures/simple_project --mode standard`
- [ ] `ctxclean fixtures/repeated_log.txt --max-tokens 120`
- [ ] `ctxclean fixtures/dirty_html_article.html --fit gpt-4.1 --format json`
- [ ] `ctxclean report fixtures/ci_failure_log.txt --max-tokens 8000 --format json`
- [ ] Create a temporary directory with `src/keep.txt` and `.env`, then verify `ctxclean <tempdir> --include-sensitive --format json` includes `.env` with `[REDACTED_SECRET]`

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

## GitHub Release

- [ ] Tag from a clean release commit: `git tag -a v0.1.0 -m "ContextClean v0.1.0"`
- [ ] Push commit and tag.
- [ ] Confirm `.github/workflows/release.yml` built artifacts for Linux, macOS, and Windows.
- [ ] Attach `ctxclean` and `ctxrun` archives plus SHA256 checksum files.
- [ ] Mark known limitations clearly.

## GitHub Topics

Add:

- `llm`
- `ai-agents`
- `context-engineering`
- `token-optimization`
- `context-window`
- `developer-tools`
- `cli`
- `rust`
- `mcp`
- `local-first`
- `log-processing`
- `html-cleaner`

## Growth

- [ ] Publish launch post from `docs/LAUNCH_POST.md`.
- [ ] Open V2 issues from `docs/V2_ISSUES.md`.
- [ ] Submit to appropriate awesome lists after the GitHub release exists.
