# ContextClean v0.1.0

ContextClean is a local-first context cleaner for AI agents. It cleans noisy HTML, logs, terminal output, and repositories before they reach an LLM.

## Highlights

- `ctxclean` CLI for files, directories, and stdin.
- `ctxclean gha` for CI/GitHub Actions failure logs.
- `ctxclean repo` for safe repository context packs.
- `ctxclean report` for token savings, noise sources, removed-section summaries, and recommended commands.
- `ctxclean mcp` stdio MCP server with `contextclean_clean` and `contextclean_report` tools.
- `ctxrun` command wrapper that passes success output through and cleans failed output while preserving the child exit code.
- Exact `o200k_base` token counting.
- `--max-tokens` and `--fit gpt-4.1|claude-sonnet|gemini-pro`.
- Secret-like value redaction by default.
- `.gitignore` and `.ctxcleanignore` aware repo scanning.
- Fixture-backed benchmark rows in `benchmarks/results.json`.

## Measured Fixtures

| Fixture | Before | After | Reduction |
|---|---:|---:|---:|
| HTML scrape | 70,571 | 5,892 | 91.7% |
| CI failure log | 75,768 | 3,200 | 95.8% |
| Provider CI mix | 17,469 | 33 | 99.8% |
| Stack trace dump | 28,189 | 1,850 | 93.4% |

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features --locked
cargo build --workspace --release --locked
```

On Windows:

```powershell
.\scripts\check.ps1
powershell -ExecutionPolicy Bypass -File .\scripts\benchmarks.ps1
```

## Known Limitations

- The first-party GitHub Action, Python wrapper, and Homebrew notes are scaffolded in-repo and should be versioned in the next release tag.
- MCP mode is stdio-only and intentionally exposes read-only clean/report tools.
- crates.io publishing requires registry credentials and final package dry-runs.
