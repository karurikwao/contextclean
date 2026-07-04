$ErrorActionPreference = "Stop"

$UserCargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if (-not (Get-Command cargo -ErrorAction SilentlyContinue) -and (Test-Path (Join-Path $UserCargoBin "cargo.exe"))) {
    $env:PATH = "$UserCargoBin;$env:PATH"
}

function Assert-LastExitCode {
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}

& cargo check --workspace --all-features --locked
Assert-LastExitCode
& cargo fmt --all -- --check
Assert-LastExitCode
& cargo clippy --workspace --all-targets --all-features -- -D warnings
Assert-LastExitCode
& cargo test --workspace --all-features --locked
Assert-LastExitCode
& cargo build --workspace --release --locked
Assert-LastExitCode
