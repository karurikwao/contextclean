# Benchmarks

This directory holds reproducible benchmark plans and fixture notes for ContextClean.

V1 benchmark targets are intentionally modest ranges. Demo metrics should stay tied to fixture commands, and formal benchmark claims must report the required fields below.

- Dirty HTML: 60-85 percent estimated token reduction.
- Dirty HTML article exports: 45-75 percent estimated token reduction.
- Repeated logs: 30-80 percent estimated token reduction.
- CI failure logs: 25-70 percent estimated token reduction.
- Mixed Markdown/text: 20-50 percent estimated token reduction.
- Simple project directories: 0-15 percent estimated token reduction.
- Noisy generated directories: 30-70 percent estimated token reduction, depending on generated content.

Benchmarks must report input tokens, output tokens, reduction percent, runtime, skipped files, warnings, and critical-content checks. Claims in the README should only use measured fixture data.
