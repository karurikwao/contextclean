# Changelog

All notable changes to ContextClean will be documented in this file.

## 0.1.0 - 2026-07-04

- Added Phase 0 product definition docs.
- Added Phase 1 Rust workspace foundation.
- Added `ctxclean` CLI skeleton.
- Added text, Markdown, and JSON renderers.
- Added basic HTML/log cleaning, token budgeting, redaction, fixtures, and tests.
- Added structure-preserving HTML conversion for headings, links, tables, lists, and code blocks.
- Added high-confidence removal for cookie banners, modals, ads, tracking blocks, and short web boilerplate.
- Added Log Crusher behavior for timestamped repeats, duplicate stack frames, safe install noise, failed test names, and final error summaries.
- Added exact token counting with the OpenAI-compatible `o200k_base` tokenizer.
- Added `--fit gpt-4.1`, `--fit claude-sonnet`, and `--fit gemini-pro` model budget presets.
- Added semantic budget packing with exact `--max-tokens` enforcement and truncation footers.
- Added `ctxclean report` with token savings, compression ratio, biggest noise sources, removed section summaries, recommended commands, and JSON/Markdown/text output.
- Added explicit safety controls: `--redact-secrets`, `--no-redact-secrets`, and `--include-sensitive`.
- Added default sensitive path skipping, sensitive scan warnings, generated-directory skips, and broader provider token redaction.
- Added workflow integrations: `ctxclean gha`, `ctxclean repo`, stdio `ctxclean mcp`, and the `ctxrun` failure-output wrapper.
- Added deterministic benchmark fixtures, measured launch results, and benchmark refresh script.
- Added launch, release, growth, and V2 issue planning assets.
- Added CI, security, contributing, and release documentation.
