# Development

## Prerequisites

- Rust 1.85 toolchain
- Git

## Setup

```bash
cd contextclean
cargo build --workspace --locked
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
cargo test --workspace --all-features --locked
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
powershell -ExecutionPolicy Bypass -File .\scripts\check.ps1
```

POSIX shell:

```bash
sh scripts/check.sh
```

## Release Build

```bash
cargo build --workspace --release --locked
```

## Cloudflare Pages

The static project site lives in `site/`. The canonical Pages URL is `https://contextclean.pages.dev/`.

```bash
npx wrangler pages project create contextclean --production-branch main
npx wrangler pages deploy site --project-name contextclean --branch main
```

After deployment, verify the live page returns HTTP 200 and still contains `Local-first context cleaner for AI agents`.
