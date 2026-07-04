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

- Exact tokenizer support for common model vocabularies.
- Stronger semantic truncation boundaries.

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

## Phase 5: Context Reports

- `ctxclean report`.
- Noise source ranking.
- Suggested command output.
- Explain/diff mode for removed content.

## Phase 6: Agent Integrations

- MCP server.
- Agent workflow examples.
- GitHub Action.
- Cursor/Codex/Claude Code integration docs.

## Phase 7: Distribution

- crates.io package.
- Cross-platform release binaries.
- Homebrew formula.
- Python wrapper.
