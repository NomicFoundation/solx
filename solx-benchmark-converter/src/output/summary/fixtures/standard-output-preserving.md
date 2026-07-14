### 🧪 Integration tests — standard · PR vs `main`

✅ **Output-preserving** — bytecode size identical (7 comparisons), solx-tester gas identical (3).
✅ **No new failures** — Foundry's 3 / Hardhat's 2 failures already present on `main`.

| Suite | New failures | Size Δ | Gas Δ | Report |
|---|---|---|---|---|
| solx-tester · 2 proj | ✅ 0 | ✅ 0 of 3 | ✅ 0 of 3 | [solx-tester-report.xlsx ↓](https://example.com/artifacts/tester) |
| Foundry · 2 proj | ✅ 0 (3 pre-existing) | ✅ 0 of 4 | ⚪ jitter 2 of 4, median 0.1% (not gated) | [foundry-report.xlsx ↓](https://example.com/artifacts/foundry) |
| Hardhat | ✅ 0 (2 pre-existing) | ⚪ not collected | ⚪ not collected | [hardhat-report.xlsx ↓](https://example.com/artifacts/hardhat) |

**Compile time** — wall-clock tripwire, positive = PR slower (authoritative Δ in `ci:compile-benchmark`)

| Suite | legacy (agg / median) | viaIR (agg / median) |
|---|---|---|
| Foundry · 2 proj | +0.0% / +0.2% | +0.3% / +0.2% |
| Hardhat | +0.6% / +0.6% | +0.2% / +0.2% |

_Within noise — no suite ≥ 5%, no project ≥ 15%._
