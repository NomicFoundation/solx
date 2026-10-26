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

The fixtures are the exact standard-JSON inputs the compiler receives from Hardhat,
captured by the hardhat repo's `bench:dump-standard-json` (the solx
regression benchmark's "Dump solx standard JSON" step) and vendored here in
the corpus layout `fixtures/<scenario>/<variant>.json`. `fixtures/manifest.json`
is the dump's provenance record (per-file sha256, hardhat commit, CI run URL,
scenario repo+commit pins) — it describes the full corpus, of which this
directory vendors the legacy-DWARF subset:

| fixture | contracts | notes |
|---|---|---|
| `ens-verifiable-factory-solx/solx-legacy-dwarf.json` | 52 | |
| `openzeppelin-contracts-0.34/solx-legacy-dwarf.json` | 422 | the heavy cell (~50 s/compile on the CI runner) |
| `uniswap-v4-core-solx/solx-legacy-dwarf.json` | 157 | |

These compile with production settings — optimizer enabled and DWARF debug
info — unlike hand-packed inputs, so timings here are comparable to what the
Hardhat solx benchmark measures (minus Hardhat's own overhead).

Not yet vendored: `aave-v4-solx` — its profile compiles through multiple
per-file-override jobs that all overwrite the same `SLANG_STANDARD_JSON_DEBUG`
path, so which job the dump captures is machine-dependent; it returns once
the hardhat-side dump captures every job. Note its dumps require
`EVM_DISABLE_MEMORY_SAFE_ASM_CHECK=1` to compile (scenario-level env in the
hardhat benchmark; env vars are not part of standard JSON) — the validation
pass in `run.sh` fails loudly if a fixture needs an env var it doesn't get.

Long-term, the corpus is published by the hardhat workflow to
`nomic-foundation-automation/hardhat-benchmark-results` under `solx-corpus/`
(main runs only); once populated, refresh by copying from a pinned commit of
that repo instead of a CI artifact.

Env vars a fixture compiles with (standard JSON has no field for them) come
from its scenario's `compilerEnv` in `manifest.json`. Fixtures over 2 MB are
timed `--large-runs` times (default 3) instead of `--runs`.

## Extending

- **More fixtures**: drop any valid standard-JSON input into a
  `fixtures/<name>/` subdirectory; `run.sh` picks up every
  `fixtures/*/*.json`. The per-binary validation pass rejects fixtures that
  don't compile clean (the standard-JSON protocol reports errors inside the
  JSON with exit code 0, so hyperfine alone would silently time failures).
