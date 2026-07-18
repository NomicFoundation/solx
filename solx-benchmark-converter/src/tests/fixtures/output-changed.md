### 🧪 Integration tests — standard · PR vs `main`

⚠️ **Output changed** — 8 of 9 size comparisons differ (+139 B total); 1 of 1 solx-tester gas comparison differs. If this PR is meant to be output-preserving, investigate before merging.
✅ **No new failures**.

| Suite | New failures | Size Δ | Gas Δ | Report |
|---|---|---|---|---|
| solx-tester | ✅ 0 | ✅ 0 of 1 | ⚠️ 1 of 1 | — |
| Foundry | ✅ 0 | ⚠️ 8 of 8 (+139 B) | ⚪ not collected | — |

**solx-tester — largest gas changes:**

- `test/libsolidity/semanticTests/structs/delete_struct.sol` [Yul M3B3 0.8.34] 85,899 → 85,902 (+0.0%)

**Foundry — largest size changes:**

- `src/C0.sol:C0` [legacy, runtime] 2,000 → 2,048 B (+2.4%)
- `src/C6.sol:C6` [legacy, deploy] 1,600 → 1,616 B (+1.0%)
- `src/C5.sol:C5` [legacy, deploy] 1,500 → 1,515 B (+1.0%)
- `src/C4.sol:C4` [legacy, deploy] 1,400 → 1,414 B (+1.0%)
- `src/C3.sol:C3` [legacy, deploy] 1,300 → 1,313 B (+1.0%)
- +3 more — see foundry-report.xlsx
