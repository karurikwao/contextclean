# ContextClean

Local-first context cleaner for AI agents.

ContextClean turns noisy HTML, logs, pasted terminal output, and project files into compact, reviewable, token-budget-aware context before it gets sent to an LLM. The CLI is `ctxclean`.

> Status: Phase 0 and Phase 1 foundation. The repo has a buildable Rust workspace, product scope, documentation, CI, fixtures, and a working CLI foundation.

## Why ContextClean?

AI agents work best when they get the right context, not the most context. Raw web pages include scripts, cookie banners, tracking markup, and repeated navigation. CI logs bury the real failure under install chatter and repeated retries. Project folders can accidentally include secrets, generated files, caches, and binaries.

ContextClean gives developers a local, deterministic first pass:

```bash
ctxclean dirty.html --mode standard --max-tokens 4000 --output clean.md
ctxclean ./fixtures/simple_project --format json
cat build.log | ctxclean --mode aggressive --format text
```

It emits cleaned content plus metrics such as estimated input tokens, output tokens, tokens saved, removed sections, warnings, and truncation status.

## Works Today

ContextClean is `jq` for LLM context: pipe in messy context, get back clean, compact, model-ready output.

The Phase 1 foundation currently includes:

- Single-command CLI: `ctxclean [OPTIONS] [INPUT]`
- File, directory, and stdin input
- `light`, `standard`, and `aggressive` cleaning modes
- Text, Markdown, and JSON output formats
- Estimated token-budget packing with `--max-tokens`
- HTML execution and boilerplate removal
- Adjacent repeated-line compression for logs
- Defensive redaction of secret-like values
- `.gitignore` and `.ctxcleanignore` aware directory scanning
- Reproducible fixtures, tests, and CI

## Planned V1 Hardening

- Exact tokenizer support for common OpenAI-compatible vocabularies
- Parser-backed HTML/Markdown cleaning
- Stronger log pattern grouping and stack trace preservation
- Context reports with noise source ranking
- More benchmark fixtures and measured README claims

## Install From Source

Rust is required. Install it from [rustup.rs](https://rustup.rs/) or use the Docker commands in `docs/DEVELOPMENT.md`.

```bash
git clone https://github.com/contextclean/contextclean
cd contextclean
cargo install --path crates/contextclean-cli
```

The crates.io package and release binaries are planned after the V1 implementation hardening pass.

Run without installing:

```bash
cargo run -p contextclean-cli -- fixtures/dirty_html_small.html --format json
```

No host Rust yet, but Docker is available:

```bash
docker run --rm -v "${PWD}:/work" -w /work rust:latest cargo test --workspace --all-features --locked
```

## Quick Start

Clean an HTML export:

```bash
ctxclean fixtures/dirty_html_small.html --mode standard --output clean.md --force
```

Compress a repeated log:

```bash
ctxclean fixtures/repeated_log.txt --mode aggressive --format text
```

Create machine-readable output:

```bash
ctxclean fixtures/dirty_html_small.html --format json
```

Fit output into a budget:

```bash
ctxclean fixtures/repeated_log.txt --max-tokens 120
```

## CLI

```text
Usage: ctxclean [OPTIONS] [INPUT]

Arguments:
  [INPUT]  File, directory, or '-' to read from stdin. If omitted, reads piped stdin.

Options:
  -o, --output, --out <OUTPUT> Write cleaned output to a file
  -t, --max-tokens <TOKENS>    Hard ceiling for estimated output tokens
  -m, --mode <MODE>            light, standard, aggressive [default: standard]
  -f, --format <FORMAT>        text, markdown, json [default: markdown]
  -c, --strip-comments         Remove obvious code comment lines
      --dry-run                Analyze without writing output files
      --no-redact-secrets      Disable defensive redaction
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
- Directory scans skip common sensitive and generated paths.
- `.gitignore` and `.ctxcleanignore` are respected for directory scans.
- Unsafe redaction opt-out requires an explicit flag.

ContextClean does not replace a security review. Always inspect context before sharing proprietary code.

## Benchmark Targets

The repo includes fixture plans in `benchmarks/`. There are no measured benchmark claims yet; V1 claims should only use measured data.

| Fixture type | Target reduction |
|---|---:|
| Dirty HTML | 60-85 percent |
| Repeated logs | 50-80 percent |
| Mixed Markdown/text | 20-50 percent |
| Directory input | 30-70 percent |

## Development

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo build --workspace --release
```

On Windows PowerShell:

```powershell
.\scripts\check.ps1
```

## Project Site

The Cloudflare Pages landing site lives in `site/` and is configured by `wrangler.toml`.

## Roadmap

- Phase 2: harden HTML/Markdown parsing and exact tokenizer support.
- Phase 3: stronger log crusher and GitHub Actions log distiller.
- Phase 4: context reports and explain/diff output.
- Phase 5: MCP server mode for AI agents.
- Phase 6: release binaries, crates.io package, and Python wrapper.

## Contributing

Issues and pull requests are welcome once the repository is published. Start with `CONTRIBUTING.md`, `docs/MVP_SCOPE.md`, and `docs/ARCHITECTURE.md`.
