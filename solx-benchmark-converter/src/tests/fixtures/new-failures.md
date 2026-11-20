### 🧪 Integration tests — standard · PR vs `main`

⚪ **No output data** — no size or gated-gas comparison had a `main` counterpart to compare against.
❌ **New failures** — Foundry: +2 build, +1 test.
⚠️ **No baseline** — Foundry: 1 run (3 failures) has no `main` baseline; its failures are not compared.

| Suite | New failures | Size Δ | Gas Δ | Report |
|---|---|---|---|---|
| Foundry · 5 proj | ❌ +2 build, +1 test (12 pre-existing), ⚪ 3 unbaselined | ⚪ not collected | ⚪ not collected | — |

**New failures (PR vs `main`):**

- Foundry: `aave` [legacy] build failures 0 → 1
- Foundry: `morpho` [viaIR] test failures 1 → 2
- Foundry: `uniswap-v4` [legacy] build failures 0 → 1
