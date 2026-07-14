### 🧪 Integration tests — standard · PR vs `main`

⚪ **No output data** — no size or gated-gas comparisons were collected.
✅ **No new failures**.
❌ **Suite errored** — solx-tester produced no usable report.
❌ **Harness error** — Foundry: benchmark data matched no recognized toolchain naming.
⚠️ **No baseline** — Hardhat: 1 runs (5 failures) have no `main` counterpart; their failures are not compared.

| Suite | New failures | Size Δ | Gas Δ | Report |
|---|---|---|---|---|
| solx-tester | ❌ no report — suite errored | — | — | — |
| Foundry | ❌ unrecognized toolchain naming | — | — | — |
| Hardhat | ✅ 0, ⚪ 5 unbaselined | ⚪ not collected | ⚪ not collected | — |

---
_Suites run the **release** solx binary. Foundry/Hardhat gas jitters run-to-run (fuzz/invariant tests, CREATE-context deploys), so it never gates._
