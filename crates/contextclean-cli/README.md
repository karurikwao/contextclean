# contextclean-cli

Command line interface for ContextClean, a local-first context cleaner for AI agents.

This package installs:

- `ctxclean`: clean files, directories, stdin, GitHub Actions logs, reports, and MCP server mode
- `ctxrun`: run a command and clean failed output while preserving the child exit code

```bash
cargo install --path crates/contextclean-cli
ctxclean gha build.log --max-tokens 3200 --format markdown
ctxclean repo . --fit gpt-4.1 --output context.md
ctxrun --max-tokens 8000 npm test
```

The CLI does not call model APIs, send telemetry, or require API keys. Secret-like values are redacted by default, and repo scans respect `.gitignore` and `.ctxcleanignore`.

Use the workspace README for source installation and release status. The crates.io publish flow is prepared but should run only after `contextclean-core` is published and the CLI package dry-run is green.
