# Product Brief

## Name

Repository: `contextclean`

CLI: `ctxclean`

## Positioning

Local-first context cleaner for AI agents.

## Product Promise

ContextClean turns noisy agent inputs such as raw HTML, logs, stack traces, pasted terminal output, and project files into compact, token-budgeted context packs with measurable token savings, local-only processing, and no API calls.

## Problem

Developers routinely feed AI tools too much low-signal context:

- raw HTML with scripts, styles, tracking, cookie banners, and navigation
- CI logs dominated by repeated retries, package install noise, and boilerplate
- project folders containing generated files, dependency folders, caches, and secrets
- long pasted terminal sessions where the real error is buried

This wastes context windows, increases LLM cost, and raises the risk of leaking sensitive data.

## Target Users

- AI-assisted developers using Codex, Claude Code, Cursor, Copilot, or ChatGPT
- Open-source maintainers who want AI-friendly debugging artifacts
- DevOps engineers who paste CI or production logs into LLMs
- Agent builders who need deterministic context pre-processing

## Core Workflows

1. Clean a scraped web page into readable Markdown.
2. Compress a noisy CI or application log while preserving the real failure.
3. Scan a project directory and produce safe, reviewable context.
4. Fit cleaned context into an explicit model budget.
5. Generate JSON metrics for automation and future reporting.

## Success Criteria

- A new user understands the value from the README in under 30 seconds.
- `ctxclean --help` works after building from source.
- `ctxclean <file>` produces useful output without configuration.
- JSON output is stable enough for automation.
- Secret-like values are redacted by default.
- The repo builds and tests in CI on Linux, macOS, and Windows.

## Open Product Questions

- Which exact tokenizer crates should V1 use for `cl100k_base` and `o200k_base`?
- Should context reports be a subcommand or a format mode?
- How much AST-aware code compression belongs in V1 versus V2?
- Which MCP client should be the first integration target?
