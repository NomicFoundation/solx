### 🧪 Integration tests — full matrix · PR vs `main`

✅ **Output-preserving** — bytecode size identical (3 comparisons), solx-tester gas identical (1).
✅ **No new failures**.

| Suite | New failures | Size Δ | Gas Δ | Report |
|---|---|---|---|---|
| solx-tester | ✅ 0 | ✅ 0 of 1 | ✅ 0 of 1 | — |
| Foundry · 7 proj | ✅ 0 | ✅ 0 of 2 | ⚪ not collected | — |

**Compile time** — wall-clock tripwire, positive = PR slower (authoritative Δ in `ci:compile-benchmark`)

| Suite | legacy (agg / median) |
|---|---|
| Foundry · 7 proj | ⚠️ **+15.3%** / +18.0% |

⚠️ **Project outliers (≥15%):** `op` legacy **+31.0%** · `proj-4` legacy **+20.0%** · `proj-3` legacy **+19.0%** · `proj-2` legacy **+18.0%** · `proj-1` legacy **+17.0%** (+1 more)

**Bytecode size — PR vs baselines** (positive = PR larger; contracts built by both only)

| Suite | Pipeline | vs solc | vs released solx |
|---|---|---|---|
| Foundry · 7 proj | legacy | -9.8% | -4.0% |
| Foundry · 7 proj | viaIR | -6.3% | -0.3% |
