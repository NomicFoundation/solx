# Compile-time benchmark

Times `solx --standard-json <fixture>` with [hyperfine](https://github.com/sharkdp/hyperfine)
over real-world standard-JSON inputs, comparing the PR build against a main
build and the latest release binary. Runs in CI behind the `ci:compile-benchmark`
PR label (`.github/workflows/compile-benchmark.yaml`) and posts the report as a
PR comment. Report-only: timings never gate the PR.

## Running locally

```bash
./tests/benchmark/run.sh \
  --bin pr=./target/release/solx \
  --bin release=path/to/released/solx \
  --out benchmark-out
python3 ./tests/benchmark/report.py --dir benchmark-out --out benchmark-out/report.md
```

The first `--bin` is the candidate; the second is the baseline for the
relative column in the report.

## Fixtures

Solc standard-JSON inputs, one compile per fixture, EVMLA (default) pipeline.
The corpus is deliberately small while it lives in-repo; the plan is to move
it to a dedicated benchmark corpus repository fed by hardhat's
`bench:dump-standard-json` (which captures the exact input solx receives from
Hardhat), and to grow it there.

Seed fixtures were generated with `pack_standard_json.py` from the same
project pins the Hardhat solx benchmark uses:

| fixture | source | notes |
|---|---|---|
| `openzeppelin.json` | `nomicfoundation/openzeppelin-contracts` @ `f72b6b46` (`contracts/`, keys prefixed `contracts/`) | 350 sources, 422 contracts |
| `uniswap-v4.json` | `anaPerezGhiglia/uniswap-v4-core` @ `ab2b22ee` (+ submodules solmate @ `4b47a190`, openzeppelin-contracts @ `dbb6104c`) | 209 sources; excludes tests/docs/certora/mocks; `src/PoolManager.sol` pragma relaxed `0.8.26` → `^0.8.26` (same transform as the Hardhat scenario's preinstall); remappings for `@openzeppelin/` and `solmate/` |

Regeneration example:

```bash
python3 tests/benchmark/pack_standard_json.py \
  --root path/to/openzeppelin-contracts/contracts --prefix contracts/ \
  --out tests/benchmark/fixtures/openzeppelin.json
```

## Extending

- **More fixtures**: drop any valid standard-JSON input into `fixtures/`;
  `run.sh` picks up every `*.json` there. Verify it compiles error-free with
  the released solx first (`solx --standard-json fixture.json`).
- **Slang v2 pipeline**: add a second timed dimension by passing the Slang
  frontend flag through an additional `--bin` entry once the pipeline accepts
  the same standard-JSON input (the driver only assembles
  `<binary> --standard-json <fixture>` command lines).
