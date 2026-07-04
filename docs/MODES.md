# Cleaning Modes

ContextClean modes are escalating contracts. Each mode is deterministic and should be idempotent enough that running it on its own output does not keep shrinking meaningful content.

## Light

Goal: remove obvious transport noise while preserving the document nearly as-is.

Allowed:

- Normalize line endings to `\n`.
- Trim/control non-printable characters.
- Collapse excessive blank lines.
- Decode common HTML entities.
- Remove `script`, `style`, and `noscript` blocks.

Not allowed:

- Do not deduplicate lines.
- Do not remove navigation, footers, comments, stack traces, URLs, or metadata unless inside removed execution/style blocks.
- Do not reorder content.

Acceptance:

- Output remains visually recognizable.
- Main textual content is preserved.
- Logs retain every event line except removed control characters.

## Standard

Goal: reduce common context bloat while preserving user-visible meaning and evidence.

Allowed:

- Everything in `light`.
- Convert basic HTML into readable Markdown-like text while preserving headings, links, paragraphs, tables, lists, inline code, and fenced code blocks.
- Remove common boilerplate: nav, footer, aside, svg, cookie banners, consent modals, newsletter prompts, ads, tracking fragments.
- Remove HTML comments.
- Collapse repeated log lines, including timestamp-varied adjacent retries, with an explicit count.
- Collapse duplicate stack frames with an explicit marker.
- Remove safe install/build noise while preserving failures.
- Preserve errors, warnings, stack traces, timestamps, paths, versions, and code fences.

Not allowed:

- Do not summarize paragraphs.
- Do not infer intent.
- Do not remove unique content merely because it looks low value.
- Do not collapse similar-but-not-identical errors.

Acceptance:

- Human-readable content remains in original order.
- Known boilerplate fixtures lose noisy blocks.
- Unique log events remain recoverable.
- Re-running `standard` is stable.

## Aggressive

Goal: maximize context density while preserving facts, decisions, and actionable technical signal.

Allowed:

- Everything in `standard`.
- Remove broader low-signal formatting and repeated disclaimers.
- Compress repeated structures with explicit markers such as `[Repeated N times] <line>` and `[Collapsed stack frames: N duplicate frames removed]`.
- Remove badge/decorative lines.
- Remove broader sponsored/related HTML blocks when they are structurally marked as boilerplate.
- Strip generated boilerplate comments when `--strip-comments` is used.

Not allowed:

- Do not invent summaries.
- Do not merge different errors into one generic error.
- Do not remove security-relevant or configuration values unless redacted by the separate secret policy.
- Do not remove all provenance from compressed logs.

Acceptance:

- Output is shorter than `standard` on repetitive inputs.
- Unique errors, warnings, file paths, commands, versions, and decisions remain represented.
- Compression markers are explicit.
