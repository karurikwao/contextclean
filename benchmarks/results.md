# ContextClean Benchmark Results

Generated with `scripts/benchmarks.ps1` using the release `ctxclean` binary and exact `o200k_base` token counts.

| Fixture | Input tokens | Output tokens | Tokens saved | Reduction | Recommended command |
|---|---:|---:|---:|---:|---|
| HTML scrape | 70571 | 5892 | 64679 | 91.7% | `ctxclean benchmarks/fixtures/html_scrape_large.html --mode standard --max-tokens 5900` |
| CI failure log | 75768 | 3200 | 72568 | 95.8% | `ctxclean gha benchmarks/fixtures/github_actions_failure_large.log --max-tokens 3200 --format markdown` |
| Provider CI mix | 17469 | 33 | 17436 | 99.8% | `ctxclean gha benchmarks/fixtures/provider_ci_mix.log --max-tokens 1600 --format markdown` |
| Stack trace dump | 28189 | 1850 | 26339 | 93.4% | `ctxclean benchmarks/fixtures/stack_trace_dump_large.log --mode standard --max-tokens 1850` |
| Small dirty HTML | 371 | 97 | 274 | 73.9% | `ctxclean fixtures/dirty_html_article.html --mode standard` |

All rows include must-keep and must-remove content checks in `benchmarks/results.json`.
