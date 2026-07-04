# Benchmarks

This directory holds reproducible benchmark fixtures and generated result artifacts for ContextClean.

Run:

```powershell
powershell -ExecutionPolicy Bypass -File ..\scripts\benchmarks.ps1
```

From the repo root:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\benchmarks.ps1
```

The script:

- generates deterministic large fixtures under `benchmarks/fixtures/`
- builds the release `ctxclean` binary
- runs exact `o200k_base` token measurements
- checks required content is preserved
- checks known noise is removed
- writes `benchmarks/results.json`
- writes `benchmarks/results.md`

Current measured launch rows:

| Fixture | Input tokens | Output tokens | Tokens saved | Reduction |
|---|---:|---:|---:|---:|
| HTML scrape | 70,571 | 5,892 | 64,679 | 91.7% |
| CI failure log | 75,768 | 3,200 | 72,568 | 95.8% |
| Provider CI mix | 17,469 | 33 | 17,436 | 99.8% |
| Stack trace dump | 28,189 | 1,850 | 26,339 | 93.4% |
| Dirty HTML article | 371 | 97 | 274 | 73.9% |

Claims in the README and site should stay tied to `benchmarks/results.json`, not hand-maintained estimates.
