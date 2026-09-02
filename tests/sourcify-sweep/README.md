# Sourcify corpus sweep

Compiles real-world Sourcify-verified contracts through `solx --standard-json`
and reports what fails, and how. Built for the Slang frontend: the corpus is
every verified solc 0.8.34 contract with evmVersion >= cancun (112,587
contracts, extracted from Sourcify's BigQuery dataset 2026-08-18), so the
failure census is a ranked list of the frontend's remaining gaps on deployed
code. Runs in CI behind the `ci:sourcify-sweep` PR label
(`.github/workflows/sourcify-sweep.yaml`) and posts the report as a PR comment.
Report-only: frontend failures never fail the run.

## Running locally

```bash
# The committed fixtures (10 contracts, one per corpus layout class):
python3 tests/sourcify-sweep/run.py --bin ./target/release/solx --out sweep.jsonl
python3 tests/sourcify-sweep/report.py --results sweep.jsonl

# The full corpus, with a released solx as the baseline leg:
python3 tests/sourcify-sweep/run.py --bin ./target/release/solx --baseline path/to/solx-0.1.8 \
  --corpus path/to/corpus --out sweep.jsonl --memory-limit-mb 6144
```

`--sample N --seed S` picks a random subset, `--shard-count/--shard-index`
a round-robin slice of the sorted corpus, `--limit N` stops early. Re-running
with the same `--out` resumes: contracts already recorded are skipped.

## What a record becomes

Each corpus file `contracts/<chain>_<address>.json` (slang corpus
format_version 1 plus the contract's original solc `settings`) is turned into
one standard-JSON input: all sources inline under their original virtual
paths, `evmVersion`, `libraries`, `remappings` and `viaIR` passed through
verbatim, optimizer left at the solx default, output selection
`evm.bytecode.object` + `evm.deployedBytecode.object`. Settings the frontend
does not implement are deliberately not stripped: the same input must be
valid for the baseline leg, and a resulting failure (an unresolved import
that a remapping would have fixed, say) is a real gap, not a harness artifact.

## Outcomes

| outcome | meaning |
|---|---|
| `ok` | no error-severity diagnostics and non-empty bytecode for at least one contract in the record's `target` source |
| `solx-fail` | the candidate fails and the baseline compiles (or no `--baseline` was given) |
| `both-fail` | both fail: a corpus artifact or a documented solx limitation (memory-unsafe assembly with stack-too-deep, recursive stackification, CALLCODE) |
| `timeout` | the candidate exceeded `--timeout` (300 s default) |

Each leg records a failure `kind` alongside the outcome, because the Slang
frontend runs in the compiler's main process and a crash there ends the whole
standard-JSON call: `error` (diagnostics in the JSON, or exit without JSON),
`panic` (Rust panic; the message is the bucket key, the location is in the
recorded stderr tail), `abort` (signal, e.g. an LLVM fatal error),
`no-bytecode` (clean exit with the target contract missing from the output),
`timeout`. `--memory-limit-mb` caps each compiler process's address space so
an OOM becomes a deterministic `abort` instead of taking the runner down.

`report.py` ranks failures by normalised signature (paths, offsets and
numbers collapsed) with example contract ids, and splits outcomes by corpus
trait (single vs multi source, remappings, viaIR, inline assembly, explicit
evmVersion).

## Corpus

`corpus-pin.txt` pins the corpus release (URL + sha256); bump it to sweep a
new corpus. The tarball unpacks to `./corpus.json` + `./contracts/*.json`; the
CI shards extract only their slice of the sorted contract list (~200 MB each)
rather than the full 4.7 GB. The extraction query and provenance live with the
hardhat-slang-solx sweep (NomicFoundation/hardhat#8538), which sweeps the same
corpus end to end through Hardhat and the solc-frontend solx.

Corpus facts that matter for reading the report: 9,180 contracts carry
remappings and 7,874 have a non-relative import that is not an exact source
key (these need remapping support); 4,260 are viaIR; 30,018 mention
`assembly`; 655 are Hardhat 3 `project/` layouts; 1,469 use URL-style source
paths. Testnets dominate (Sepolia 41.5k, mainnet 4,750); filter by `chain_id`
locally for a production-only subset.
