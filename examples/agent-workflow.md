# Agent Workflow Examples

## Claude, Cursor, Codex, ChatGPT

Clean a repo slice before pasting it into an agent:

```bash
ctxclean repo . --max-tokens 12000 --output context.md --force
```

Explain why a log is noisy before asking for a fix:

```bash
ctxclean report build.log --max-tokens 8000 --format text
```

Run tests through `ctxrun`; success output stays raw, failure output becomes compact context:

```bash
ctxrun --max-tokens 8000 npm test
ctxrun --format markdown pytest
```

## Codex-Oriented Loop

```bash
ctxrun --max-tokens 8000 cargo test --workspace --all-features --locked
ctxclean repo crates/contextclean-core --max-tokens 10000 --output core-context.md --force
ctxclean report benchmarks/fixtures/github_actions_failure_large.log --format markdown
```

## Sensitive Repos

Default behavior skips `.env`, private keys, credential dirs, generated files, and caches.

```bash
ctxclean repo . --format json
```

Only opt into sensitive paths when you explicitly need them, and redaction still runs by default:

```bash
ctxclean repo . --include-sensitive --format json
```
