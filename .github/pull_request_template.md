## Summary

What changed and why?

## Testing

- [ ] `cargo check --workspace --all-features --locked`
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `cargo build --workspace --release`

## Documentation

- [ ] README/docs updated if the CLI contract, output schema, mode behavior, or security stance changed.

## Risk

- What can break?
- Does this alter redaction, ignore handling, output schemas, or file IO?

## Rollback

- How should this change be reverted or disabled if it misbehaves?

## Security

- [ ] No real secrets were added to tests, docs, examples, fixtures, or logs.
