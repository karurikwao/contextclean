# Development

## Prerequisites

- Rust stable toolchain
- Git

## Setup

```bash
git clone https://github.com/contextclean/contextclean
cd contextclean
cargo build --workspace
```

## Run

```bash
cargo run -p contextclean-cli -- fixtures/dirty_html_small.html
cargo run -p contextclean-cli -- fixtures/repeated_log.txt --format json
```

## Test

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

If Rust is not installed on the host but Docker is available:

```bash
docker run --rm -v "${PWD}:/work" -w /work -e CARGO_TARGET_DIR=/tmp/contextclean-target -e RUSTUP_TOOLCHAIN=1.85.0 rust:1.85-bookworm cargo check --workspace --all-features --locked
docker run --rm -v "${PWD}:/work" -w /work -e CARGO_TARGET_DIR=/tmp/contextclean-target rust:latest cargo fmt --all -- --check
docker run --rm -v "${PWD}:/work" -w /work -e CARGO_TARGET_DIR=/tmp/contextclean-target rust:latest cargo clippy --workspace --all-targets --all-features -- -D warnings
docker run --rm -v "${PWD}:/work" -w /work -e CARGO_TARGET_DIR=/tmp/contextclean-target rust:latest cargo test --workspace --all-features --locked
docker run --rm -v "${PWD}:/work" -w /work -e CARGO_TARGET_DIR=/tmp/contextclean-target rust:latest cargo build --workspace --release --locked
```

## Local Check Scripts

PowerShell:

```powershell
.\scripts\check.ps1
```

POSIX shell:

```bash
./scripts/check.sh
```

## Release Build

```bash
cargo build --workspace --release
```

## Cloudflare Pages

The static project site lives in `site/`.

```bash
npx wrangler pages project create contextclean --production-branch main
npx wrangler pages deploy site --project-name contextclean --branch main
```
