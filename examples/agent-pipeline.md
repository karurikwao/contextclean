# Agent Pipeline Example

```bash
curl -s https://example.com > page.html
ctxclean page.html --mode standard --max-tokens 4000 --output prompt-context.md --force
```

Then paste `prompt-context.md` into an AI agent prompt after reviewing it locally.
