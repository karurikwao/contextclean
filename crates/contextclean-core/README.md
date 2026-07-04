# contextclean-core

Core cleaning, scanning, reporting, rendering, and token-budgeting engine for ContextClean.

ContextClean is a local-first context cleaner for AI agents. The core crate provides:

- exact `o200k_base` token counting
- text/Markdown/JSON render models
- HTML and log cleanup
- token budget packing
- context reports
- repo-aware scanning with ignore and sensitive-path defaults
- secret-like value redaction

Most users should install the CLI package:

```bash
cargo install --path crates/contextclean-cli
```

Use the workspace README for source installation and release status. The crates.io publish flow is prepared but should run only after registry credentials and final dry-runs are green.
