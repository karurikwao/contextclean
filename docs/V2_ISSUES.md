# V2 Issue Status

These GitHub issue tracks were opened after launch and are now represented in the repository.

## Completed Tracks

| Issue | Track | Implementation |
|---|---|---|
| #8 | Parser-backed HTML/Markdown cleaner | `crates/contextclean-core/src/html.rs`, `fixtures/malformed_browser_export.html` |
| #9 | Provider-specific CI log distillers | `crates/contextclean-core/src/logs.rs`, `benchmarks/fixtures/provider_ci_mix.log` |
| #10 | Streaming `ctxrun` capture | `crates/contextclean-cli/src/bin/ctxrun.rs`, `docs/commands.md` |
| #11 | MCP compatibility matrix | `docs/MCP_COMPATIBILITY.md`, `examples/mcp-client.json` |
| #12 | First-party GitHub Action wrapper | `action.yml`, `.github/workflows/action-smoke.yml`, `examples/github-actions.yml` |
| #13 | Homebrew formula or tap instructions | `docs/HOMEBREW.md` |
| #14 | Python package wrapper | `packages/python`, `examples/langchain_helper.py`, `examples/llamaindex_helper.py` |

## Remaining Follow-Ups

- Tag a new release so the GitHub Action can be referenced by an immutable version instead of `@main`.
- Publish the Homebrew tap from `docs/HOMEBREW.md`.
- Publish the Python package after packaging credentials and release policy are ready.
- Add more real-world fixtures for provider-specific CI logs and malformed browser exports.
