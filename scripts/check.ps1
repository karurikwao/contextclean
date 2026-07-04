$ErrorActionPreference = "Stop"

function Invoke-Cargo {
    cargo @args
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}

Invoke-Cargo check --workspace --all-features --locked
Invoke-Cargo fmt --all -- --check
Invoke-Cargo clippy --workspace --all-targets --all-features -- -D warnings
Invoke-Cargo test --workspace --all-features --locked
Invoke-Cargo build --workspace --release --locked
