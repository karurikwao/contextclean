# Output Schemas

All renderers use the same internal result model. JSON is canonical. Text and Markdown are human-oriented projections.

## Shared Model

```json
{
  "version": "0.1.0",
  "mode": "standard",
  "format": "json",
  "source": "input.html",
  "input": {
    "bytes": 18420,
    "chars": 18102,
    "tokens": 4526
  },
  "output": {
    "bytes": 6920,
    "chars": 6811,
    "tokens": 1703,
    "content": "Cleaned context..."
  },
  "metrics": {
    "input_tokens": 4526,
    "output_tokens": 1703,
    "tokens_saved": 2823,
    "compression_ratio": 0.376,
    "reduction_percent": 62.4
  },
  "truncation": {
    "applied": false,
    "limit_tokens": null,
    "tokens_removed": 0,
    "reason": null
  },
  "removed_sections": [],
  "noise_sources": [],
  "warnings": [],
  "metadata": {
    "elapsed_ms": 4,
    "engine": "contextclean-core"
  }
}
```

## Nested Object Shapes

`removed_sections` is an array of:

```json
{
  "kind": "html_boilerplate",
  "label": "HTML boilerplate block: nav",
  "tokens_removed": 120,
  "count": 1
}
```

Allowed `removed_sections.kind` values:

- `html_execution_block`
- `html_boilerplate`
- `html_comment`
- `duplicate_line`
- `stack_frame`
- `log_noise`
- `code_comment`
- `secret`
- `truncated`
- `other`

`noise_sources` is an array of:

```json
{
  "kind": "repetition",
  "label": "repeated log lines",
  "tokens_removed": 420
}
```

Allowed `noise_sources.kind` values:

- `html_boilerplate`
- `repetition`
- `stack_trace`
- `log_noise`
- `code_comments`
- `secret`
- `truncation`
- `other`

`warnings` is an array of:

```json
{
  "code": "secrets_redacted",
  "message": "redacted 1 secret-like value(s)",
  "severity": "warning"
}
```

Allowed `warnings.severity` values:

- `info`
- `warning`
- `error`

Nullable fields:

- `source`
- `truncation.limit_tokens`
- `truncation.reason`

Budget note:

- `--max-tokens` budgets cleaned `output.content`.
- Text and Markdown renderers also cap their full rendered output to the estimated budget.
- JSON output remains valid pretty-printed JSON; the full envelope is not capped, but `output.content` and `output.tokens` reflect the budget.

## Text

Text output contains cleaned content followed by a compact metrics footer.

```text
<cleaned content>

---
ctxclean v0.1.0
mode: Standard
format: Text
input_tokens: 4526
output_tokens: 1703
tokens_saved: 2823
reduction_percent: 62.4%
truncation: false
```

## Markdown

Markdown output is designed for copying into prompts or issue comments. It includes:

- `# Cleaned Context`
- cleaned content
- metrics table
- removed section table when available
- warning table when available

## JSON

JSON output must be valid JSON on stdout with no human diagnostics mixed in. Diagnostics belong on stderr.
