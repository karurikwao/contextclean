# contextclean Python Wrapper

This package is a thin typed wrapper around the local `ctxclean` binary. It does not reimplement the Rust engine; it shells out to the CLI so Python workflows use the same token counting, redaction, safety rules, and JSON schemas.

Install the CLI first:

```bash
cargo install --git https://github.com/karurikwao/contextclean contextclean-cli
```

Use from Python:

```python
from contextclean import clean_file, output_text, report

cleaned = clean_file("build.log", max_tokens=8000, output_format="json")
print(output_text(cleaned))

summary = report("./project", output_format="json")
print(summary["tokens"])
```

Helpers:

- `clean_text(text, ...)`
- `clean_file(path, ...)`
- `clean_github_actions_log(path, ...)`
- `report(path, ...)`
- `output_text(result)`

The wrapper raises `ContextCleanError` when the underlying process exits non-zero.
