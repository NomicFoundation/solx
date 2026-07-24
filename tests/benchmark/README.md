# Compile-time benchmark

Times `solx --standard-json <fixture>` with [hyperfine](https://github.com/sharkdp/hyperfine)
over real-world standard-JSON inputs, comparing the PR build against a main
build and the latest release binary. Runs in CI behind the `ci:compile-benchmark`
PR label (`.github/workflows/compile-benchmark.yaml`) and posts the report as a
PR comment. Report-only: timings never gate the PR.

## Running locally

The corpus is not vendored — fetch it first (a sparse checkout keeps it to the
handful of fixtures the benchmark uses), then point `run.sh` at it:

```bash
git clone --depth 1 --filter=blob:none --sparse --branch solx-corpus-preview \
  https://github.com/nomic-foundation-automation/hardhat-benchmark-results corpus
git -C corpus sparse-checkout set solx-corpus

./tests/benchmark/run.sh \
  --bin pr=./target/release/solx \
  --bin release=path/to/released/solx \
  --fixtures corpus/solx-corpus \
  --out benchmark-out
python3 ./tests/benchmark/report.py --dir benchmark-out --out benchmark-out/report.md
```

The first `--bin` is the candidate; the second is the baseline for the
relative column in the report. By default only the EVMLA (legacy) pipeline is
timed; add `--variant solx-via-ir-dwarf` to also benchmark the via-IR pipeline.

## Fixtures

The fixtures are the exact standard-JSON inputs solx receives from Hardhat,
captured by the hardhat repo's `bench:dump-standard-json` (the solx regression
benchmark's "Dump solx standard JSON" step) and published to the corpus repo
`nomic-foundation-automation/hardhat-benchmark-results` in the layout
`solx-corpus/<scenario>/<variant>.json` (with `solx-corpus/manifest.json`
recording per-file sha256, hardhat commit, CI run URL, and scenario repo+commit
pins). The workflow checks it out at a pinned commit — see the "Checkout
benchmark corpus" step; bump that `ref` to refresh the fixtures.

> This is a **temporary preview** location (`solx-corpus-preview`, force-pushed
> per hardhat PR run — hence pinning to a commit, not the branch tip). The
> final home is `solx-corpus/` on that repo's `main`, published from hardhat
> merge runs.

The benchmark times two variants per scenario, both compiled with production
settings (optimizer enabled + DWARF debug info) so timings are comparable to
what the Hardhat solx benchmark measures (minus Hardhat's own overhead):

- `solx-legacy-dwarf` — the EVMLA pipeline, timed by default.
- `solx-via-ir-dwarf` — the via-IR pipeline, off by default (enabled by the
  `include_via_ir` `workflow_dispatch` input, or `--variant` locally).

| scenario | contracts | notes |
|---|---|---|
| `ens-verifiable-factory-solx` | 52 | |
| `openzeppelin-contracts-0.34` | 422 | the heavy cell (~50 s/compile on the CI runner) |
| `uniswap-v4-core-solx` | 157 | |

`aave-v4-solx` exists in the corpus but is not benchmarked: its dumps require
`EVM_DISABLE_MEMORY_SAFE_ASM_CHECK=1` to compile (scenario-level env in the
hardhat benchmark; env vars are not part of standard JSON), so the validation
pass in `run.sh` would abort on it. The "Checkout benchmark corpus" step
therefore doesn't fetch it.

## Extending

- **More fixtures**: publish the standard-JSON dump to the corpus repo, then
  add its `solx-corpus/<scenario>/<variant>.json` path to the sparse-checkout
  list in the "Checkout benchmark corpus" workflow step (and a `--variant` if
  it introduces a new variant name). `run.sh`'s per-binary validation pass
  rejects fixtures that don't compile clean — the standard-JSON protocol
  reports errors inside the JSON with exit code 0, so hyperfine alone would
  silently time failures.
- **Slang v2 pipeline**: add a second timed dimension by passing the Slang
  frontend flag through an additional `--bin` entry once the pipeline accepts
  the same standard-JSON input (the driver only assembles
  `<binary> --standard-json <fixture>` command lines).
