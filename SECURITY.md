# Security Policy

## Supported Versions

ContextClean is pre-1.0. Security fixes apply to the latest unreleased main branch until release artifacts exist.

## Trust Model

ContextClean is designed to run locally. The current local CLI contains no telemetry, remote sync, hosted service, or model API calls.

## Secret Handling

- Secret-like values are redacted by default.
- Common sensitive files and generated folders are skipped in directory scans.
- `.gitignore` and `.ctxcleanignore` are respected for directory scans.
- `--no-redact-secrets` is an explicit unsafe override.

## Reporting A Vulnerability

Use GitHub private vulnerability reporting when the public repository is available. If private reporting is not enabled yet, contact the repository owner privately through GitHub and include:

- affected version or commit
- operating system
- exact command or input pattern
- impact and expected exposure
- a minimal reproduction that does not contain real secrets

Do not open public issues containing secrets or exploit details.
