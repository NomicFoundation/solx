### 🧪 Integration tests — full matrix · PR vs `main`

✅ **Output-preserving** — bytecode size identical (3 comparisons), solx-tester gas identical (1).
✅ **No new failures**.

| Suite | New failures | Size Δ | Gas Δ | Report |
|---|---|---|---|---|
| solx-tester | ✅ 0 | ✅ 0 of 1 | ✅ 0 of 1 | — |
| Foundry · 2 proj | ✅ 0 | ✅ 0 of 2 | ⚪ not collected | — |

**Compile time** — wall-clock tripwire, positive = PR slower (authoritative Δ in `ci:compile-benchmark`)

| Suite | legacy (agg / median) |
|---|---|
| Foundry · 2 proj | ⚠️ **+10.8%** / +15.9% |

⚠️ **Project outliers (>15%):** `op` legacy **+31.0%**

**Bytecode size — PR vs baselines** (positive = PR larger; contracts built by both only)

| Suite | Pipeline | vs solc | vs released solx |
|---|---|---|---|
| Foundry · 2 proj | legacy | -9.8% | -4.0% |
| Foundry · 2 proj | viaIR | -6.3% | -0.3% |

---
_Suites run the **release** solx binary. Foundry/Hardhat gas jitters run-to-run (fuzz/invariant tests, CREATE-context deploys), so it never gates._
