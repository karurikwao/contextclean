# Basic Clean Example

```bash
ctxclean fixtures/dirty_html_small.html --mode standard --output clean.md --force
```

Expected result:

- script/style blocks removed
- navigation/footer boilerplate removed
- main article content preserved
- metrics included in Markdown output
