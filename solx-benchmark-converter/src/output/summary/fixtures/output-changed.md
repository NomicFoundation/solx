### 🧪 Integration tests — standard · PR vs `main`

⚠️ **Output changed** — 7 of 8 size comparisons differ (+91 B total); 1 of 1 solx-tester gas comparisons differ. If this PR is meant to be output-preserving, investigate before merging.
✅ **No new failures**.

| Suite | New failures | Size Δ | Gas Δ | Report |
|---|---|---|---|---|
| solx-tester | ✅ 0 | ✅ 0 of 1 | ⚠️ 1 of 1 | — |
| Foundry | ✅ 0 | ⚠️ 7 of 7 (+91 B) | ⚪ not collected | — |

**solx-tester — largest gas changes:**

- `test/libsolidity/semanticTests/structs/delete_struct.sol` [Yul M3B3 0.8.34] 85,899 → 85,902 (+0.0%)

**Foundry — largest size changes:**

- `src/C6.sol:C6` [legacy, deploy] 1,600 → 1,616 B (+1.0%)
- `src/C5.sol:C5` [legacy, deploy] 1,500 → 1,515 B (+1.0%)
- `src/C4.sol:C4` [legacy, deploy] 1,400 → 1,414 B (+1.0%)
- `src/C3.sol:C3` [legacy, deploy] 1,300 → 1,313 B (+1.0%)
- `src/C2.sol:C2` [legacy, deploy] 1,200 → 1,212 B (+1.0%)
- +2 more — full list in foundry-report.xlsx
