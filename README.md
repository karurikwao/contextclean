# ContextClean

Local-first context cleaner for AI agents.

ContextClean turns noisy HTML, logs, pasted terminal output, and project files into compact, reviewable, token-budget-aware context before it gets sent to an LLM. The CLI is `ctxclean`.

> Status: V1 foundation with Phase 3-7 behavior in place: HTML cleaning, log crushing, exact token counting, model budget fitting, context reports, and repo-safety defaults.

## Why ContextClean?

AI agents work best when they get the right context, not the most context. Raw web pages include scripts, cookie banners, tracking markup, and repeated navigation. CI logs bury the real failure under install chatter and repeated retries. Project folders can accidentally include secrets, generated files, caches, and binaries.

ContextClean gives developers a local, deterministic first pass:

```bash
ctxclean dirty.html --mode standard --max-tokens 4000 --output clean.md
ctxclean ./fixtures/simple_project --format json
cat build.log | ctxclean --mode aggressive --format text
```

It emits cleaned content plus exact token metrics, budget metadata, removed sections, warnings, and truncation status.

## Works Today

ContextClean is `jq` for LLM context: pipe in messy context, get back clean, compact, model-ready output.

The current foundation includes:

- Single-command CLI: `ctxclean [OPTIONS] [INPUT]`
- File, directory, and stdin input
- `light`, `standard`, and `aggressive` cleaning modes
- Text, Markdown, and JSON output formats
- Exact OpenAI-compatible token counting through the `o200k_base` tokenizer
- Token-budget packing with `--max-tokens`
- Model presets with `--fit gpt-4.1`, `--fit claude-sonnet`, and `--fit gemini-pro`
- `ctxclean report` for token savings, biggest noise sources, removed section summaries, and recommended commands
- Structure-preserving HTML cleanup for headings, links, paragraphs, tables, and code blocks
- HTML execution, modal, ad, tracking, cookie, nav, footer, aside, and SVG removal
- Log crushing for repeated lines, timestamped retries, duplicate stack frames, install noise, and failure preservation
- Defensive redaction of secret-like values, enabled by default
- `.gitignore` and `.ctxcleanignore` aware directory scanning
- Default skips for `.git`, `node_modules`, build outputs, caches, sensitive dot-directories, `.env`, private keys, tokens, and certificate-like files
- Reproducible fixtures, tests, and CI

## Planned V1 Hardening

- Parser-backed HTML/Markdown cleaning for malformed and deeply nested pages
- Broader log pattern grouping for more CI providers
- More benchmark fixtures and measured README claims

## Install From Source

Rust is required. Install it from [rustup.rs](https://rustup.rs/) or use the Docker commands in `docs/DEVELOPMENT.md`.

```bash
cargo install --path crates/contextclean-cli
```

The crates.io package and release binaries are planned after the V1 implementation hardening pass.

Run without installing:

```bash
cargo run -p contextclean-cli -- fixtures/dirty_html_small.html --format json
```

No host Rust yet, but Docker is available:

```bash
docker run --rm -v "${PWD}:/work" -w /work -e CARGO_TARGET_DIR=/tmp/contextclean-target rust:1.85-bookworm sh -lc 'export PATH=/usr/local/cargo/bin:$PATH; cargo test --workspace --all-features --locked'
```

## Quick Start

Clean an HTML export:

```bash
ctxclean fixtures/dirty_html_article.html --mode standard --output clean.md --force
```

Compress a repeated log:

```bash
ctxclean fixtures/ci_failure_log.txt --mode standard --format text
```

Create machine-readable output:

```bash
ctxclean fixtures/dirty_html_small.html --format json
```

Fit output into a budget:

```bash
ctxclean fixtures/repeated_log.txt --max-tokens 120
```

Fit output for a model preset:

```bash
ctxclean fixtures/dirty_html_article.html --fit gpt-4.1 --format markdown
```

Explain why a file is noisy:

```bash
ctxclean report fixtures/ci_failure_log.txt --max-tokens 8000 --format markdown
```

Opt into sensitive files only when you mean it:

```bash
ctxclean ./project-with-local-env --include-sensitive --format json
```

## CLI

```text
Usage: ctxclean [OPTIONS] [INPUT]
       ctxclean report [OPTIONS] <INPUT>

Arguments:
  [INPUT]  File, directory, or '-' to read from stdin. If omitted, reads piped stdin.

Options:
  -o, --output, --out <OUTPUT> Write cleaned output to a file
  -t, --max-tokens <TOKENS>    Hard ceiling for output content tokens
      --fit <FIT>              gpt-4.1, claude-sonnet, gemini-pro
  -m, --mode <MODE>            light, standard, aggressive [default: standard]
  -f, --format <FORMAT>        text, markdown, json [default: markdown]
  -c, --strip-comments         Remove obvious code comment lines
      --dry-run                Analyze without writing output files
      --redact-secrets         Keep default defensive redaction enabled explicitly
      --no-redact-secrets      Disable defensive redaction
      --include-sensitive      Include sensitive paths such as .env and private keys
      --force                  Overwrite output file if it exists
  -q, --quiet                  Suppress non-error diagnostics
  -v, --verbose                Print extra diagnostics
  -h, --help                   Show help
  -V, --version                Show version
```

## Sample JSON Shape

```json
{
  "version": "0.1.0",
  "mode": "standard",
  "format": "json",
  "source": "input.html",
  "input": { "bytes": 18420, "chars": 18102, "tokens": 4526 },
  "output": { "bytes": 6920, "chars": 6811, "tokens": 1703, "content": "..." },
  "metrics": {
    "input_tokens": 4526,
    "output_tokens": 1703,
    "tokens_saved": 2823,
    "compression_ratio": 0.376,
    "reduction_percent": 62.4
  },
  "budget": {
    "fit": null,
    "model_id": null,
    "tokenizer": "o200k_base",
    "token_count_is_exact": true,
    "preset_limit_tokens": null,
    "effective_limit_tokens": null,
    "model_max_output_tokens": null,
    "limit_source": "none"
  },
  "truncation": {
    "applied": false,
    "limit_tokens": null,
    "tokens_removed": 0,
    "reason": null
  },
  "removed_sections": [],
  "noise_sources": [],
  "warnings": [],
  "metadata": { "elapsed_ms": 4, "engine": "contextclean-core" }
}
```

## Trust And Privacy

ContextClean is designed to run locally.

- No telemetry or network calls are part of the V1 foundation.
- Secret-like values are redacted by default before output.
- Directory scans skip common sensitive and generated paths by default.
- `.gitignore` and `.ctxcleanignore` are respected for directory scans.
- Sensitive files and credential directories require `--include-sensitive`.
- `--include-sensitive` does not override `.gitignore` or `.ctxcleanignore`.
- Unsafe redaction opt-out requires `--no-redact-secrets`.

ContextClean does not replace a security review. Always inspect context before sharing proprietary code.

## Benchmark Targets

The repo includes fixture plans in `benchmarks/`. Demo metrics should stay tied to fixture commands, and public benchmark claims should only use measured data.

| Fixture type | Target reduction |
|---|---:|
| Dirty HTML | 60-85 percent |
| Dirty HTML article exports | 45-75 percent |
| Repeated logs | 30-80 percent |
| CI failure logs | 25-70 percent |
| Mixed Markdown/text | 20-50 percent |
| Simple project directories | 0-15 percent |
| Noisy generated directories | 30-70 percent |

## Development

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features --locked
cargo build --workspace --release --locked
```

On Windows PowerShell:

```powershell
.\scripts\check.ps1
```

## Project Site

The Cloudflare Pages landing site lives in `site/` and is configured by `wrangler.toml`.

## Roadmap

- Phase 2: exact tokenizer support and stronger semantic truncation. Implemented with `o200k_base` counting and semantic boundary packing.
- Phase 3: HTML and Markdown cleaner. Implemented with parser-light deterministic conversion; parser-backed hardening remains planned.
- Phase 4: Log Crusher. Implemented for repeated lines, duplicate stack frames, install noise, and failure preservation.
- Phase 5: token budget packer. Implemented with `--max-tokens`, `--fit`, exact counts, semantic truncation, and truncation footers.
- Phase 6: context reports. Implemented with `ctxclean report`.
- Phase 7: safety and repo awareness. Implemented with ignore handling, default skips, sensitive-path warnings, redaction, and explicit sensitive opt-in.

Future phases: MCP server mode, release binaries, crates.io package, Python wrapper, and provider-specific CI log distillers.

## Contributing

Issues and pull requests are welcome once the repository is published. Start with `CONTRIBUTING.md`, `docs/MVP_SCOPE.md`, and `docs/ARCHITECTURE.md`.
