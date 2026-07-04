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

## Future: Agent Integrations

- MCP server.
- Agent workflow examples.
- GitHub Action.
- Cursor/Codex/Claude Code integration docs.

## Future: Distribution

- crates.io package.
- Cross-platform release binaries.
- Homebrew formula.
- Python wrapper.
