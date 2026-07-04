# Launch Post Draft

## Short Version

ContextClean is a local-first CLI that cleans noisy AI context before you paste it into Claude, Cursor, Codex, ChatGPT, or an MCP-capable agent.

It removes raw web junk, crushes repeated CI logs, redacts secret-like values by default, respects `.gitignore` and `.ctxcleanignore`, and fits output into exact token budgets.

```bash
ctxclean gha build.log --max-tokens 3200
ctxclean repo . --fit gpt-4.1 --output context.md
ctxrun --max-tokens 8000 npm test
```

## Longer Version

AI agents do better work when they get the right context, not the most context. But the context developers paste every day is usually full of junk: scripts and cookie banners from web pages, install noise from CI logs, repeated stack frames, generated files, local caches, and sometimes secret-shaped values.

ContextClean is a deterministic local cleaner for that layer.

- HTML scrape fixture: 70,571 -> 5,874 tokens
- CI failure fixture: 75,768 -> 3,200 tokens
- Stack trace fixture: 28,189 -> 1,850 tokens

Those numbers are generated from committed fixtures with exact `o200k_base` token counts and content checks.

It includes:

- `ctxclean gha failed-log.txt`
- `ctxclean repo .`
- `ctxclean report ./project --max-tokens 8000`
- `ctxclean mcp`
- `ctxrun npm test`

The design is local-first: no telemetry, no API keys, no network calls in the CLI. Redaction is on by default, and sensitive paths require explicit opt-in.

## Suggested Tags

`rust`, `cli`, `ai-agents`, `llm`, `context-engineering`, `mcp`, `developer-tools`, `local-first`
