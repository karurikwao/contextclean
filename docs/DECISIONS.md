# Decisions

## ADR-0001: Rust Workspace

Decision: Use Rust for the CLI and core engine.

Reason: ContextClean is a text-processing CLI where deterministic performance, portable binaries, and low runtime overhead matter.

## ADR-0002: CLI Command Is `ctxclean`

Decision: The repo name is `contextclean`, and the executable is `ctxclean`.

Reason: The repository name is descriptive, while the CLI command is short enough for pipelines.

## ADR-0003: JSON Is The Canonical Output Schema

Decision: Render text, Markdown, and JSON from the same internal result model.

Reason: This keeps automation stable while allowing user-friendly terminal output.

## ADR-0004: Local-Only V1 Foundation

Decision: V1 foundation contains no network calls, telemetry, hosted service, or model API calls.

Reason: Trust and privacy are core adoption drivers for context tooling.

## ADR-0005: Exact Local Token Counting

Decision: Use local `o200k_base` token counting for metrics and budget enforcement.

Reason: Phase 1 started with deterministic estimates to keep the foundation buildable. Phase 5 now requires exact, explainable budget packing, and `o200k_base` gives ContextClean an OpenAI-compatible baseline without model API calls.
