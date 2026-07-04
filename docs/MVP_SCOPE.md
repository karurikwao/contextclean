# MVP Scope

## V1 Features

- Rust workspace with `contextclean-core` and `contextclean-cli`.
- CLI command named `ctxclean`.
- Input from file, directory, stdin, or `-`.
- Output to stdout or `--output`.
- `--mode light|standard|aggressive`.
- `--format text|markdown|json`.
- `--max-tokens` using deterministic estimated token budgeting.
- `--strip-comments` for obvious line comments.
- HTML execution block removal: `script`, `style`, and `noscript`.
- Standard HTML boilerplate removal: `nav`, `footer`, `aside`, `svg`, comments, cookie/newsletter/ad/modal/tracking blocks, and short high-confidence boilerplate lines.
- Structure-preserving HTML conversion for headings, links, paragraphs, tables, lists, inline code, and fenced code blocks.
- Log Crusher compression for repeated lines, timestamped retries, duplicate stack frames, and safe install/build noise.
- Preservation of unique errors, failed test names, failure summaries, stack roots, and timestamps around failures.
- Secret-like value redaction by default.
- `.gitignore` and `.ctxcleanignore` aware directory scanning.
- Default skips for sensitive/generated paths.
- Tests for CLI startup, output formats, cleaner behavior, redaction, and directory safety.
- CI for format, clippy, test, and release build.

## V1 Non-Goals

- No GUI or TUI.
- No cloud service, accounts, telemetry, or hosted storage.
- No remote repository cloning.
- No editor extension.
- No model API calls.
- No AI-generated summaries.
- No perfect semantic compression guarantee.
- No AST-aware code understanding yet.
- No fully parser-backed HTML DOM cleanup yet.
- No provider-specific CI log parser yet.
- No exact tokenizer guarantee until tokenizer crates are selected.
- No plugin system.
- No background daemon or file watcher.
- No mutation of input files.
- No guarantee that all secrets are detected.

## Mode Definitions

See `docs/MODES.md` for the full contract.

## Output Schemas

See `docs/OUTPUT_SCHEMAS.md` for text, Markdown, and JSON output contracts.

## Phase 0 Acceptance Criteria

- Final name and CLI command are documented.
- One-sentence positioning is documented.
- V1 feature list and non-goals are explicit.
- Mode definitions are testable.
- Output schemas are documented.
- Benchmark targets are documented.

## Phase 1 Acceptance Criteria

- Rust workspace exists.
- CLI crate and core crate exist.
- README, license, security, contributing, changelog, and docs exist.
- GitHub Actions CI exists.
- Fixtures and tests exist.
- `cargo test --workspace --all-features --locked` passes locally, in CI, or through the documented Docker verification path when host Rust is unavailable.
