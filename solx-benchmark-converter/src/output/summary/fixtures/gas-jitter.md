### 🧪 Integration tests — standard · PR vs `main`

✅ **Output-preserving** — bytecode size identical (3 comparisons).
✅ **No new failures**.

| Suite | New failures | Size Δ | Gas Δ | Report |
|---|---|---|---|---|
| Foundry | ✅ 0 | ✅ 0 of 2 | ⚪ jitter 2 of 2, median <0.1% (not gated) | — |
| Hardhat | ✅ 0 | ✅ 0 of 1 | ⚪ no jitter (not gated) | — |
