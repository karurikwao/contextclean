# Benchmark Plan

Benchmarks must be reproducible and conservative.

## Fixtures

- `fixtures/dirty_html_small.html`
- `fixtures/dirty_html_article.html`
- `fixtures/repeated_log.txt`
- `fixtures/ci_failure_log.txt`
- `fixtures/mixed_markdown.md`
- `fixtures/simple_project/`

Future fixture groups:

- dirty HTML medium and large
- provider-specific GitHub Actions logs
- stack trace dumps

## Required Metrics

- input bytes, chars, estimated tokens
- output bytes, chars, estimated tokens
- tokens saved
- reduction percent
- runtime
- warnings
- truncation state
- removed section categories

## Targets

| Fixture | Target |
|---|---:|
| Dirty HTML | 60-85 percent reduction |
| Dirty HTML article exports | 45-75 percent reduction |
| Repeated logs | 30-80 percent reduction |
| CI failure logs | 25-70 percent reduction |
| Mixed Markdown/text | 20-50 percent reduction |
| Simple project directories | 0-15 percent reduction |
| Noisy generated directories | 30-70 percent reduction |
