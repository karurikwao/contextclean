# Architecture

ContextClean is a Rust workspace with a thin CLI crate and a reusable core crate.

```text
crates/
  contextclean-cli/   argument parsing, IO, exit codes, user-facing behavior
  contextclean-core/  cleaning, HTML conversion, log crushing, scanning, rendering, schemas, errors
```

## Core Flow

1. Read source from file, directory, stdin, or `-`.
2. Apply ignore rules and default skip rules for directory input.
3. Clean text according to mode.
4. Redact secret-like values unless explicitly disabled.
5. Apply exact token budget if requested.
6. Render text, Markdown, or JSON.
7. For `ctxclean report`, project the clean result into report metrics instead of rendering cleaned content.
8. For `ctxclean mcp`, expose clean/report through stdio JSON-RPC without human logs on stdout.
9. For `ctxrun`, pass successful child output through unchanged and clean failed output through the same core pipeline.

## Current Implementation Boundaries

- Token counts use the OpenAI-compatible `o200k_base` tokenizer through `tiktoken-rs`.
- Model presets map `gpt-4.1`, `claude-sonnet`, and `gemini-pro` to conservative local budgets.
- HTML handling is deterministic and parser-backed: high-confidence block removal, DOM rendering for malformed/browser-exported fragments, and Markdown-like conversion for common article structures.
- Log crushing is deterministic: safe install noise removal, duplicate frame collapse, and repeated-line grouping.
- Directory traversal uses the `ignore` crate to respect `.gitignore` and `.ctxcleanignore`.
- Sensitive path scanning is explicit opt-in with `--include-sensitive`; redaction stays enabled by default.
- `ctxclean gha` and `ctxclean repo` are CLI aliases over the same safe source reader and cleaner.
- `ctxclean mcp` is stdio-only and exposes read-only clean/report tools.
- `ctxrun` is a second binary in the CLI crate for failed command output.
- No network calls, telemetry, remote storage, or model API calls exist in the V1 foundation.

## Future Architecture

Later phases should add:

- deeper parser-backed HTML/Markdown coverage for more browser export variants
- more provider-specific CI log fixtures and distillers
- interactive TTY streaming mode for `ctxrun`
- language-aware code compression behind explicit flags
