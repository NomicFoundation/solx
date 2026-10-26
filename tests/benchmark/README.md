# Compile-time benchmark

Times `slang --standard-json <fixture>` with [hyperfine](https://github.com/sharkdp/hyperfine)
over real-world standard-JSON inputs, comparing the PR merged into its base
branch against a main build and a pinned prerelease binary. Runs in CI on every
same-repo PR that changes code (`.github/workflows/compile-benchmark.yaml`) and
posts the report as a PR comment. Report-only: timings never gate the PR.

## Running locally

```bash
./tests/benchmark/run.sh \
  --bin pr=./target/release/slang \
  --bin main=path/to/main/slang \
  --out benchmark-out
python3 ./tests/benchmark/report.py --dir benchmark-out --out benchmark-out/report.md
```

The first `--bin` is the candidate; the second is the baseline for the
relative column in the report.

## Fixtures

The fixtures are the exact standard-JSON inputs slang receives from Hardhat in the
scenarios of the hardhat repo's slang regression benchmark, one file per compilation
job, captured by `bench:dump-standard-json --compiler slang`. A full dispatch of the
"Slang Regression Benchmark" workflow uploads them as the `slang-standard-json`
artifact; refresh by copying that artifact over `fixtures/`. `fixtures/manifest.json`
is the dump's provenance record: hardhat commit, run URL, scenario repo and commit
pins, per-file sha256, and the env vars each scenario compiles with, which `run.sh`
sets because standard JSON has no field for them.

| scenario | jobs | contracts | notes |
|---|---|---|---|
| `1inch-aqua-solx` | 1 | 57 | |
| `aave-v4-solx` | 3 | 211 | src only; `EVM_DISABLE_MEMORY_SAFE_ASM_CHECK` |
| `ens-verifiable-factory-solx` | 1 | 62 | |
| `graph-horizon-solx` | 1 | 253 | `EVM_DISABLE_MEMORY_SAFE_ASM_CHECK` |
| `lidofinance-core-solx` | 2 | 166 | src only; `EVM_DISABLE_MEMORY_SAFE_ASM_CHECK` |
| `openzeppelin-contracts-0.34` | 1 | 841 | |
| `solady-solx` | 1 | 314 | |
| `uniswap-v4-core-solx` | 1 | 174 | |

These compile with the settings Hardhat ships (optimizer enabled, DWARF debug info,
debug symbols), so timings here are comparable to what the Hardhat slang benchmark
measures, minus Hardhat's own overhead.

Fixtures over 2 MB are timed `--large-runs` times (default 3) instead of `--runs`.

## Extending

- **More fixtures**: drop any valid standard-JSON input into a
  `fixtures/<name>/` subdirectory and add `<name>` under `scenarios` in
  `manifest.json`; `run.sh` picks up every `fixtures/*/*.json`. The per-binary validation pass rejects fixtures that
  don't compile clean (the standard-JSON protocol reports errors inside the
  JSON with exit code 0, so hyperfine alone would silently time failures).
