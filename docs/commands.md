# Commands

The CLI exposes the default cleaning command plus a report subcommand:

```bash
ctxclean [OPTIONS] [INPUT]
ctxclean report [OPTIONS] <INPUT>
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
