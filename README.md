# ContextClean

![MSRV](https://img.shields.io/badge/MSRV-1.85-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Local First](https://img.shields.io/badge/local--first-no%20telemetry-brightgreen)
![MCP](https://img.shields.io/badge/MCP-stdio%20server-purple)

Local-first context cleaner for AI agents.

ContextClean turns noisy HTML, CI logs, stack traces, pasted terminal output, and project folders into compact, redacted, token-budgeted context before it reaches an LLM. The CLI is `ctxclean`; the failure-log wrapper is `ctxrun`.

```bash
ctxclean gha build.log --max-tokens 3200 --format markdown
ctxclean repo . --fit gpt-4.1 --output context.md
ctxrun --max-tokens 8000 npm test
```

## Proof Points

Measured with `scripts/benchmarks.ps1`, the release `ctxclean` binary, and exact `o200k_base` token counts. Each row has must-keep and must-remove checks in `benchmarks/results.json`.

| Fixture | Before | After | Saved | Reduction |
|---|---:|---:|---:|---:|
| HTML scrape | 70,571 | 5,892 | 64,679 | 91.7% |
| CI failure log | 75,768 | 3,200 | 72,568 | 95.8% |
| Provider CI mix | 17,469 | 33 | 17,436 | 99.8% |
| Stack trace dump | 28,189 | 1,850 | 26,339 | 93.4% |
| Dirty HTML article | 371 | 97 | 274 | 73.9% |

## Why ContextClean?

AI agents work best when they get the right context, not the most context. Raw pages carry scripts, cookie banners, tracking blocks, and repeated navigation. CI logs bury the actual failure under install chatter and repeated retries. Repo scans can accidentally include generated files, caches, local env files, and secret-shaped values.

ContextClean gives you a deterministic local first pass:

- exact OpenAI-compatible token counting with `o200k_base`
- model budgets through `--max-tokens` and `--fit gpt-4.1|claude-sonnet|gemini-pro`
- HTML cleanup that preserves headings, links, paragraphs, tables, lists, and code blocks
- parser-backed malformed HTML cleanup that keeps nested lists, tables, and code readable
- provider-specific log crushing for GitHub Actions, pytest, cargo, Docker Buildx, Playwright, pnpm, and npm noise
- repo-aware scanning that respects `.gitignore` and `.ctxcleanignore`
- secret-like value redaction enabled by default
- reports that explain tokens saved, noise sources, removed sections, and the next command to run
- stdio MCP mode for agent workflows
- streaming `ctxrun` for cleaning failed command output while preserving the child exit code

## Install

Download a release binary from [GitHub Releases](https://github.com/karurikwao/contextclean/releases/tag/v0.1.0).
Linux x64, macOS Intel, macOS Apple Silicon, and Windows x64 archives are attached.

Install from Git with Rust 1.85 or newer:

```bash
cargo install --git https://github.com/karurikwao/contextclean contextclean-cli
```

Install from a local checkout:

```bash
git clone https://github.com/karurikwao/contextclean.git
cd contextclean
cargo install --path crates/contextclean-cli
```

No host Rust yet, but Docker is available:

```bash
docker run --rm -v "${PWD}:/work" -w /work -e CARGO_TARGET_DIR=/tmp/contextclean-target rust:1.85-bookworm sh -lc 'export PATH=/usr/local/cargo/bin:$PATH; cargo test --workspace --all-features --locked'
```

crates.io publishing is prepared for `v0.1.0`; see `docs/RELEASE_CHECKLIST.md`.

## Quick Start

Clean an HTML export:

```bash
ctxclean fixtures/dirty_html_article.html --mode standard --output clean.md --force
```

Clean a GitHub Actions failure log:

```bash
ctxclean gha fixtures/ci_failure_log.txt --max-tokens 8000 --format markdown
```

Pack a repository safely:

```bash
ctxclean repo . --max-tokens 12000 --format markdown
```

Explain why context is noisy:

```bash
ctxclean report benchmarks/fixtures/github_actions_failure_large.log --max-tokens 8000 --format text
```

Run a test command and only clean the output when it fails:

```bash
ctxrun --max-tokens 8000 pytest
ctxrun --format markdown npm test
```

Start MCP server mode:

```bash
ctxclean mcp
```

## Commands

```text
ctxclean [OPTIONS] [INPUT]
ctxclean gha [OPTIONS] <INPUT>
ctxclean repo [OPTIONS] <INPUT>
ctxclean report [OPTIONS] <INPUT>
ctxclean mcp
ctxrun [OPTIONS] <COMMAND> [ARGS]...
```

Core options:

| Option | Description |
|---|---|
| `-o, --output, --out <PATH>` | Write cleaned output or reports to a file |
| `-t, --max-tokens <TOKENS>` | Hard ceiling for cleaned content tokens |
| `--fit <MODEL>` | `gpt-4.1`, `claude-sonnet`, or `gemini-pro` |
| `-m, --mode <MODE>` | `light`, `standard`, or `aggressive` |
| `-f, --format <FORMAT>` | `text`, `markdown`, or `json` |
| `-c, --strip-comments` | Remove obvious code comment lines |
| `--redact-secrets` | Keep default redaction enabled explicitly |
| `--no-redact-secrets` | Disable defensive redaction |
| `--include-sensitive` | Include sensitive paths only when explicitly requested |
| `--dry-run` | Print without writing output files |
| `--force` | Overwrite output files |
| `-q, --quiet` | Suppress non-error diagnostics |
| `-v, --verbose` | Print extra diagnostics to stderr |

## Integrations

- `ctxclean gha failed-log.txt`: GitHub Actions-focused log cleanup. Defaults to aggressive mode.
- `ctxclean repo .`: safe repo context packer using the same ignore and redaction pipeline as the default cleaner.
- `ctxclean mcp`: newline-delimited JSON-RPC MCP server over stdio with `contextclean_clean` and `contextclean_report` tools.
- `ctxrun npm test`: command wrapper that passes successful output through unchanged, cleans failed output, and exits with the child exit code.
- First-party GitHub Action wrapper: `uses: karurikwao/contextclean@main`.
- MCP compatibility matrix: `docs/MCP_COMPATIBILITY.md`.
- Homebrew direct-install and tap notes: `docs/HOMEBREW.md`.
- Python wrapper scaffold: `packages/python`.
- Examples for Claude/Cursor/Codex, GitHub Actions, MCP clients, LangChain, and LlamaIndex live in `examples/`.

## JSON Shape

`--format json` emits a canonical `CleanResult` with stable top-level fields:

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

Report JSON uses `tokens`, `biggest_noise_sources`, `removed_section_summary`, `recommended_command`, and `warnings`, without including the cleaned body.

## Trust And Privacy

ContextClean is designed to run locally.

- No telemetry or network calls are part of the CLI.
- Secret-like values are redacted by default before output.
- Directory scans skip common sensitive and generated paths by default.
- `.gitignore` and `.ctxcleanignore` are respected for directory scans.
- Sensitive files and credential directories require `--include-sensitive`.
- `--include-sensitive` does not override ignore files.
- Unsafe redaction opt-out requires `--no-redact-secrets`.

ContextClean does not replace a security review. Always inspect context before sharing proprietary code.

## Benchmarks

Refresh benchmark fixtures and measured tables:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\benchmarks.ps1
```

Outputs:

- `benchmarks/fixtures/`
- `benchmarks/results.json`
- `benchmarks/results.md`

## Comparison

| Tooling pattern | What it gives you | ContextClean difference |
|---|---|---|
| `cat file | llm` | Fast raw context | Cleans, redacts, measures, and budgets before paste/send |
| Log tailing | Recent failure text | Collapses repeated noise and preserves unique failures |
| HTML-to-Markdown converters | Readable pages | Removes AI-hostile boilerplate and reports token savings |
| Manual prompt trimming | Human judgment | Deterministic, reproducible, exact token accounting |
| Cloud preprocessors | Convenience | Local-first, no API keys, no telemetry |

## Roadmap

- V0.1.0: HTML cleaner, log crusher, token budgets, reports, repo safety, MCP stdio, `ctxrun`, launch benchmarks.
- V0.1.x: parser-backed malformed HTML handling, provider CI distillers, streaming `ctxrun`, first-party action, MCP matrix, Homebrew direct-install notes, and Python wrapper scaffold.
- V0.2.0: versioned GitHub Action tag, published Homebrew tap, shell completions, and richer release binaries.
- V0.3.0: published Python package and deeper LangChain/LlamaIndex adapters.
- V0.4.0: AST-aware code compression behind explicit flags.

## Good First Issues

- Add a real-world GitHub Actions fixture from a public failing workflow.
- Add more malformed HTML fixtures from browser exports and docs sites.
- Add shell completions for Bash, Zsh, Fish, and PowerShell.
- Turn `docs/HOMEBREW.md` into a published tap.
- Validate more MCP clients and update `docs/MCP_COMPATIBILITY.md`.
- Add benchmark fixtures for Docker Buildx, pytest, cargo, and Playwright logs.

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

## Contributing

Issues and pull requests are welcome once the repository is published. Start with `CONTRIBUTING.md`, `docs/MVP_SCOPE.md`, `docs/ARCHITECTURE.md`, and `docs/ROADMAP.md`.
