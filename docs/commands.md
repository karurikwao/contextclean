# Commands

The CLI exposes the default cleaning command, workflow-specific aliases, report mode, MCP server mode, and the `ctxrun` failure wrapper:

```bash
ctxclean [OPTIONS] [INPUT]
ctxclean gha [OPTIONS] <INPUT>
ctxclean repo [OPTIONS] <INPUT>
ctxclean report [OPTIONS] <INPUT>
ctxclean mcp
ctxrun [OPTIONS] <COMMAND> [ARGS]...
```

## Inputs

- file path
- directory path
- `-` for stdin
- omitted input for piped stdin

## Options

| Option | Description |
|---|---|
| `-o, --output, --out <PATH>` | Write output to a file |
| `-t, --max-tokens <N>` | Fit cleaned content within an exact token budget |
| `--fit <MODEL>` | Use a model preset: `gpt-4.1`, `claude-sonnet`, or `gemini-pro` |
| `-m, --mode <MODE>` | `light`, `standard`, or `aggressive` |
| `-f, --format <FORMAT>` | `text`, `markdown`, or `json` |
| `-c, --strip-comments` | Remove obvious code comment lines |
| `--dry-run` | Analyze and print output without writing output files |
| `--redact-secrets` | Keep default redaction enabled explicitly |
| `--no-redact-secrets` | Disable secret redaction |
| `--include-sensitive` | Allow sensitive paths such as `.env`, private keys, credential dirs, and certificate-like files when they are not ignored |
| `--force` | Overwrite output file |
| `-q, --quiet` | Suppress non-error diagnostics |
| `-v, --verbose` | Print extra diagnostics |
| `-h, --help` | Show help |
| `-V, --version` | Show version |

`--max-tokens` must be at least 5, the shortest budget that can preserve `[Context Truncated]`. Token counts use the OpenAI-compatible `o200k_base` tokenizer. Text and Markdown renderers cap the rendered output to the requested budget. JSON keeps stdout parseable and includes the full pretty-printed envelope, while `output.content` and `output.tokens` remain budgeted.

`--fit` sets the default budget to a known model preset:

| Preset | Model id in JSON | Budget |
|---|---|---:|
| `gpt-4.1` | `gpt-4.1` | 1,047,576 |
| `claude-sonnet` | `claude-sonnet-5` | 1,000,000 |
| `gemini-pro` | `gemini-2.5-pro` | 1,048,576 |

`--fit` and `--max-tokens` can be combined when `--max-tokens` is less than or equal to the preset.

## Reports

`ctxclean report <INPUT>` cleans the input internally, then prints an explanation instead of cleaned content.

```bash
ctxclean report build.log --max-tokens 8000
ctxclean report ./project --format json
```

Reports include:

- input tokens
- output tokens
- tokens saved
- compression ratio
- biggest noise sources
- removed section summary
- recommended cleanup command
- warnings from repo scanning or redaction

## GitHub Actions Logs

`ctxclean gha <INPUT>` is a log-focused alias for GitHub Actions and similar CI output.

```bash
ctxclean gha failed-log.txt --max-tokens 8000 --format markdown
ctxclean gha - --format text
```

It defaults to `aggressive` mode so install/build chatter, repeated retries, and duplicate stack frames are collapsed while failed test names, unique errors, and final summaries are preserved.

## Repository Context

`ctxclean repo <PATH>` is an explicit repo/project packer.

```bash
ctxclean repo . --max-tokens 12000 --format markdown
ctxclean repo ./crates/contextclean-core --format json
```

It reuses the same directory reader as the default command: `.gitignore`, global git excludes, `.ctxcleanignore`, generated-directory skips, sensitive-path warnings, and default redaction all apply.

## MCP Server Mode

`ctxclean mcp` starts a stdio JSON-RPC MCP server. Stdout contains protocol messages only.

Supported methods:

- `initialize`
- `notifications/initialized`
- `ping`
- `tools/list`
- `tools/call`
- `shutdown`

Tools:

- `contextclean_clean`
- `contextclean_report`

Tool arguments accept `text` or `path`, plus `mode`, `format`, `maxTokens`, `fit`, `stripComments`, `redactSecrets`, and `includeSensitive`.

## ctxrun

`ctxrun` executes a command locally. Successful commands pass stdout/stderr through unchanged. Failed commands are cleaned with ContextClean, printed to stdout, and the process exits with the child command's exit code.

```bash
ctxrun --max-tokens 8000 npm test
ctxrun --format markdown pytest
ctxrun --max-tokens 6000 cargo test -p contextclean-core
```

`ctxrun` options apply only to cleaned failure output; success output is not redacted or rewritten.

## Safety Flags

Directory scans respect `.gitignore`, global git excludes, and `.ctxcleanignore`. They also skip generated directories such as `.git`, `node_modules`, `target`, `dist`, `build`, cache directories, and common virtual environments.

Sensitive paths are skipped by default and produce warnings during directory scans. Explicit sensitive file or directory input fails unless `--include-sensitive` is supplied. Redaction still runs by default after opt-in inclusion. `--include-sensitive` does not override `.gitignore`, global git excludes, or `.ctxcleanignore`.

## Exit Codes

| Code | Meaning |
|---:|---|
| 0 | success |
| 1 | processing/rendering failure |
| 2 | usage, missing input, or config error |
| 3 | filesystem read/write error |
