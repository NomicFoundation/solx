### 🧪 Integration tests — standard · PR vs `main`

✅ **Output-preserving** — bytecode size identical (1 comparisons), solx-tester gas identical (1).
✅ **No new failures**.

| Suite | New failures | Size Δ | Gas Δ | Report |
|---|---|---|---|---|
| solx-tester | ✅ 0 | ✅ 0 of 1 | ✅ 0 of 1 | — |

---
_Suites run the **release** solx binary. Foundry/Hardhat gas jitters run-to-run (fuzz/invariant tests, CREATE-context deploys), so it never gates._
