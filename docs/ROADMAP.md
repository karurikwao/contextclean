# Roadmap

## Phase 0: Product Definition

- Name and CLI command selected.
- Product promise documented.
- V1 feature list and non-goals documented.
- Output schemas, modes, and benchmark targets documented.

## Phase 1: Repository Foundation

- Buildable Rust workspace.
- Professional open-source file set.
- CI workflow.
- Tests and fixtures.
- Working CLI foundation.

## Phase 2: Token And Truncation Hardening

- Exact token counting through `o200k_base`.
- Stronger semantic truncation boundaries.
- Model preset aliases for `gpt-4.1`, `claude-sonnet`, and `gemini-pro`.

## Phase 3: HTML And Markdown Cleaner

- Structure-preserving conversion for headings, links, paragraphs, tables, lists, inline code, and fenced code blocks.
- High-confidence removal for script/style/svg/nav/footer/aside, cookie banners, modals, ads, tracking blocks, and comments.
- Aggressive mode removes broader sponsored/related HTML boilerplate.
- Parser-backed DOM cleanup remains future hardening.

## Phase 4: Log Crusher

- Pattern-aware repeated log grouping.
- Duplicate stack frame collapse.
- Safe install/build noise removal.
- Failed test, final error, timestamp, and stack-root preservation.
- GitHub Actions log distiller remains future hardening.

## Phase 5: Token Budget Packer

- `--max-tokens`.
- `--fit gpt-4.1|claude-sonnet|gemini-pro`.
- Semantic truncation at paragraph and code boundaries.
- Footer explaining removed tokens and target budget.

## Phase 6: Context Reports

- `ctxclean report`.
- Noise source ranking.
- Suggested command output.
- Removed section summary.

## Phase 7: Safety And Repo Awareness

- `.gitignore` and `.ctxcleanignore` support.
- Generated directory skips for `.git`, `node_modules`, `target`, `dist`, `build`, caches, and virtual environments.
- Sensitive path skips and warnings by default.
- `--include-sensitive` explicit opt-in.
- Secret redaction enabled by default.

## Phase 8: Integrations

- `ctxclean gha` for CI/GitHub Actions failure logs.
- `ctxclean repo` for explicit safe repository context packs.
- `ctxclean mcp` stdio JSON-RPC MCP server with clean/report tools.
- `ctxrun` command wrapper for failed test/build output.
- Claude/Cursor/Codex, GitHub Actions, MCP, LangChain, and LlamaIndex examples.

## Phase 9: Benchmarks And Launch Assets

- Deterministic large benchmark fixture generation.
- `benchmarks/results.json` and `benchmarks/results.md`.
- README proof table, comparison table, roadmap, and good-first-issue list.
- Cloudflare Pages launch copy aligned with measured fixtures.

## Phase 10: Release And Growth

- Release checklist and release notes prepared for `v0.1.0`.
- Release workflow draft for cross-platform artifacts.
- GitHub topics, launch post, awesome-list submission notes, and V2 issue seeds.

## Future: Distribution

- crates.io package publish after registry credentials and package dry-runs are green.
- Cross-platform release binaries attached to the GitHub release after a remote is configured.
- Homebrew formula.
- Python wrapper.
- Provider-specific CI log distillers.
- Parser-backed HTML/Markdown cleaner hardening.
