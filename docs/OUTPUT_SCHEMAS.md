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
  "budget": {
    "fit": null,
    "model_id": null,
    "tokenizer": "o200k_base",
    "token_count_is_exact": true,
    "preset_limit_tokens": null,
    "effective_limit_tokens": null,
    "model_max_output_tokens": null,
    "limit_source": "none"
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
- `budget.fit`
- `budget.model_id`
- `budget.preset_limit_tokens`
- `budget.effective_limit_tokens`
- `budget.model_max_output_tokens`
- `truncation.limit_tokens`
- `truncation.reason`

Budget object:

- `tokenizer` is currently `o200k_base`.
- `token_count_is_exact` is `true`.
- `limit_source` is `none`, `max_tokens`, `fit`, or `fit_and_max_tokens`.
- `fit` is `gpt-4.1`, `claude-sonnet`, `gemini-pro`, or `null`.

Budget note:

- `--max-tokens` budgets cleaned `output.content`.
- `--fit` supplies a preset budget when `--max-tokens` is omitted.
- Semantic truncation prefers paragraph and code-block boundaries and appends a footer when content is removed.
- Text and Markdown renderers also cap their full rendered output to the requested budget.
- JSON output remains valid pretty-printed JSON; the full envelope is not capped, but `output.content` and `output.tokens` reflect the budget.

Truncation footer:

```text
[Context Truncated: Removed 14203 tokens to fit 8000-token budget]
```

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

## Report JSON

`ctxclean report <INPUT> --format json` emits a report envelope without the cleaned content body:

```json
{
  "version": "0.1.0",
  "source": "build.log",
  "mode": "standard",
  "format": "json",
  "budget": {
    "fit": null,
    "model_id": null,
    "tokenizer": "o200k_base",
    "token_count_is_exact": true,
    "preset_limit_tokens": null,
    "effective_limit_tokens": 8000,
    "model_max_output_tokens": null,
    "limit_source": "max_tokens"
  },
  "tokens": {
    "input": 12000,
    "output": 6200,
    "saved": 5800,
    "compression_ratio": 0.516,
    "reduction_percent": 48.3
  },
  "biggest_noise_sources": [
    {
      "kind": "repetition",
      "label": "repeated log lines",
      "tokens_removed": 4200
    }
  ],
  "removed_section_summary": [
    {
      "kind": "duplicate_line",
      "count": 1284,
      "tokens_removed": 4200
    }
  ],
  "recommended_command": "ctxclean build.log --max-tokens 8000 --format markdown",
  "warnings": []
}
```

Report Markdown contains `Token Summary`, `Biggest Noise Sources`, `Removed Section Summary`, and `Recommended Command` sections. It adds `Warnings` when scanner or redaction warnings exist.
