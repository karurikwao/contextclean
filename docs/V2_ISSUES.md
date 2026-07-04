# V2 Issue Seeds

Use these as copy-ready GitHub issues after the repository is published.

## Parser-Backed HTML/Markdown Cleaner

Implement parser-backed cleanup for malformed, deeply nested, and browser-exported HTML while preserving the current deterministic output contracts.

Acceptance:

- malformed HTML fixture coverage
- nested table/list/code preservation
- no regression in current HTML fixtures
- output schemas unchanged

## Provider-Specific CI Log Distillers

Add targeted cleanup rules for GitHub Actions, pytest, cargo, Docker Buildx, Playwright, pnpm, and npm logs.

Acceptance:

- provider fixtures in `benchmarks/fixtures`
- must-keep failed tests and final summaries
- must-remove install/build noise
- benchmark results refreshed

## Streaming ctxrun Capture

Replace `Command::output()` with bounded concurrent stdout/stderr capture to avoid pipe deadlocks and support long-running commands.

Acceptance:

- success output remains raw
- failure output is cleaned
- child exit code is preserved
- byte cap and timeout behavior documented

## MCP Compatibility Matrix

Verify `ctxclean mcp` with Claude Desktop, Cursor, Codex, VS Code MCP clients, and other common stdio MCP clients.

Acceptance:

- client config examples
- initialize/tools/list/tools/call smoke notes
- sensitive-path and redaction behavior verified

## GitHub Action Wrapper

Create a first-party action that can clean failed logs and upload compact context artifacts.

Acceptance:

- `action.yml`
- example workflow
- no secret leakage in artifacts
- integration test or smoke workflow

## Homebrew Formula

Prepare a Homebrew formula or tap instructions for release binaries.

Acceptance:

- install command documented
- checksum flow documented
- upgrade path documented

## Python Package

Create a thin Python wrapper for invoking `ctxclean` and parsing JSON reports.

Acceptance:

- package scaffold
- typed helper functions
- LangChain/LlamaIndex examples updated
