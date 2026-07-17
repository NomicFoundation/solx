### 🧪 Integration tests — standard · PR vs `main`

⚪ **No output data** — no size or gated-gas comparison had a `main` counterpart to compare against.
❌ **New failures** — Foundry: +2 build, +1 test.

| Suite | New failures | Size Δ | Gas Δ | Report |
|---|---|---|---|---|
| Foundry · 5 proj | ❌ +2 build, +1 test (12 pre-existing) | ⚪ not collected | ⚪ not collected | — |

**New failures (PR vs `main`):**

- Foundry: `aave` [legacy] build failures 0 → 1
- Foundry: `morpho` [viaIR] test failures 1 → 2
- Foundry: `uniswap-v4` [legacy] build failures 0 → 1
