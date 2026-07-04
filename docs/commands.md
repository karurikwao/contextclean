# Commands

Phase 1 exposes a single command:

```bash
ctxclean [OPTIONS] [INPUT]
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
| `-t, --max-tokens <N>` | Fit output within an estimated token budget |
| `-m, --mode <MODE>` | `light`, `standard`, or `aggressive` |
| `-f, --format <FORMAT>` | `text`, `markdown`, or `json` |
| `-c, --strip-comments` | Remove obvious code comment lines |
| `--dry-run` | Analyze and print output without writing output files |
| `--no-redact-secrets` | Disable secret redaction |
| `--force` | Overwrite output file |
| `-q, --quiet` | Suppress non-error diagnostics |
| `-v, --verbose` | Print extra diagnostics |

## Exit Codes

| Code | Meaning |
|---:|---|
| 0 | success |
| 1 | processing/rendering failure |
| 2 | usage, missing input, or config error |
| 3 | filesystem read/write error |
