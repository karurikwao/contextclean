# Benchmark Methodology

Benchmarks are reproducible and fixture-backed.

## Refresh Results

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\benchmarks.ps1
```

The script generates large fixtures, builds the release binary, runs `ctxclean --format json --quiet`, parses exact token metrics, validates critical content checks, and writes:

- `benchmarks/results.json`
- `benchmarks/results.md`

## Current Launch Fixtures

| Fixture | Command |
|---|---|
| HTML scrape | `ctxclean benchmarks/fixtures/html_scrape_large.html --mode standard --max-tokens 5900 --format json --quiet` |
| CI failure log | `ctxclean benchmarks/fixtures/github_actions_failure_large.log --mode aggressive --max-tokens 3200 --format json --quiet` |
| Stack trace dump | `ctxclean benchmarks/fixtures/stack_trace_dump_large.log --mode standard --max-tokens 1850 --format json --quiet` |
| Dirty HTML article | `ctxclean fixtures/dirty_html_article.html --mode standard --format json --quiet` |

## Required Metrics

- input bytes, chars, exact tokens
- output bytes, chars, exact tokens
- tokens saved
- reduction percent
- warnings
- truncation state
- removed section categories
- must-contain checks
- must-not-contain checks

## Guardrails

- Public claims must cite generated fixture results.
- Runtime claims should use warmed release runs before publication.
- Fixtures should not contain real secrets.
- Budgeted rows must preserve the declared must-contain checks.
- Any changed cleaner behavior should rerun the benchmark script before README/site updates.
