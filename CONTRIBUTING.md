# Contributing

Thanks for helping make ContextClean sharper.

## Local Setup

```bash
cargo build --workspace
cargo test --workspace --all-features
```

## Before Opening A Pull Request

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

## Pull Request Expectations

- Keep changes scoped.
- Add or update tests for behavior changes.
- Update docs when CLI contracts or output schemas change.
- Do not add network calls, telemetry, or external services without a documented decision.
- Do not include real secrets in fixtures.

## Good First Issues

Good first tasks should be small, testable, and linked to docs or fixtures. Examples:

- add a fixture
- improve help text
- document a workflow
- add an output assertion
- improve redaction tests
