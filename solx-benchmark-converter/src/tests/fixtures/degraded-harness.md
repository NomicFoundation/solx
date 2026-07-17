### 🧪 Integration tests — standard · PR vs `main`

⚪ **No output data** — no size or gated-gas comparison had a `main` counterpart to compare against.
⚪ **No failure data** — no PR run had a `main` counterpart to compare against.
❌ **Suite errored** — solx-tester produced no usable report.
❌ **Harness error** — Foundry: benchmark data matched no recognized toolchain naming.
❌ **Harness error** — Hardhat: runs matched no declared toolchain: `04.mason-legacy`.
⚠️ **No baseline** — Hardhat: 1 run (5 failures) has no `main` counterpart; its failures are not compared.

| Suite | New failures | Size Δ | Gas Δ | Report |
|---|---|---|---|---|
| solx-tester | ❌ no report — suite errored | — | — | [solx-tester-report.xlsx ↓](https://example.com/artifacts/tester) |
| Foundry | ❌ unrecognized toolchain naming | — | — | — |
| Hardhat | ⚪ not compared, ⚪ 5 unbaselined | ⚪ not collected | ⚪ not collected | — |
