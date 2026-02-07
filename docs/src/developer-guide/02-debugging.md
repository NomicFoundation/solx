# Debugging and Inspecting Compiler Output

This guide shows how to use **solx** debug flags to inspect intermediate representations at each compilation stage.

## IR Dump Flags

Each flag writes files to the output directory (`-o`):

| Flag | Extension | Description |
|---|---|---|
| `--evmla` | `.evmla` | EVM legacy assembly from solc (legacy pipeline only) |
| `--ethir` | `.ethir` | EthIR (translated from EVM assembly, legacy pipeline only) |
| `--ir` / `--ir-optimized` | `.yul` | Yul IR from solc (via-ir pipeline only) |
| `--emit-llvm-ir` | `.unoptimized.ll`, `.optimized.ll` | LLVM IR before and after optimization |
| `--asm` | `.asm` | Final EVM assembly |

The `--debug-info` and `--debug-info-runtime` flags are output selectors that print deploy and runtime debug info to stdout (or to files when `-o` is used). They are not IR dump flags.

Example:

```bash
solx contract.sol -o ./debug --evmla --ethir --emit-llvm-ir --asm --overwrite
```

This produces one file per contract per stage in `./debug/`.

## Quick Dump with `SOLX_OUTPUT_DIR`

Setting the `SOLX_OUTPUT_DIR` environment variable enables all IR dumps at once without listing individual flags:

```bash
export SOLX_OUTPUT_DIR=./ir_dumps
solx contract.sol
```

This writes all applicable IR files for every contract, with automatic overwrite. Which files are produced depends on the pipeline used: the Yul pipeline dumps Yul and LLVM IR, while the legacy pipeline dumps EVMLA, EthIR, and LLVM IR.

## Benchmarking

The `--benchmarks` flag prints timing information for each pipeline stage:

```bash
solx contract.sol --benchmarks
```

Output includes per-contract compilation timing in milliseconds.

## LLVM Diagnostics

Two flags control LLVM-level diagnostics:

- `--llvm-verify-each` — runs LLVM IR verification after every optimization pass. Useful for catching miscompilations. Silent on success; only reports errors when verification fails.
- `--llvm-debug-logging` — enables detailed LLVM pass execution logging to stderr. Shows which passes and analyses run, with instruction counts.

```bash
solx contract.sol --llvm-verify-each --llvm-debug-logging
```

## LLVM Options Pass-Through

Arbitrary LLVM backend options can be passed with `--llvm-options`:

```bash
solx contract.sol --llvm-options='-evm-metadata-size 10'
```

The value must be a single string following `=`. See the [LLVM Options](./03-llvm-options.md) guide for available options, including EVM backend options and standard LLVM diagnostic options like `-time-passes` and `-stats`.

## Optimization Levels

**solx** maps optimization levels to LLVM pipelines:

| Flag | Middle-end | Size level | Back-end |
|---|---|---|---|
| `-O1` | Less | Zero | Less |
| `-O2` | Default | Zero | Default |
| `-O3` (default) | Aggressive | Zero | Aggressive |
| `-Os` | Default | S | Aggressive |
| `-Oz` | Default | Z | Aggressive |

The default is `-O3`, optimizing for runtime performance.

The optimization level can also be set with the `SOLX_OPTIMIZATION` environment variable (values: `1`, `2`, `3`, `s`, `z`).

## Size Fallback

The `--optimization-size-fallback` flag (or `SOLX_OPTIMIZATION_SIZE_FALLBACK` env var) recompiles with `-Oz` when bytecode exceeds the 24,576-byte EVM contract size limit (EIP-170). When triggered, output files include a `.size_fallback` suffix.

## Spill Area Suffix

When the compiler uses a memory spill region to mitigate stack-too-deep errors, output files include an `.o{offset}s{size}` suffix indicating the spill area parameters. For example: `MyContract.o256s1024.ethir`.

## Typical Debugging Workflow

1. **Reproduce** the issue with a minimal Solidity file.
2. **Dump all IRs** using `SOLX_OUTPUT_DIR`:
   ```bash
   SOLX_OUTPUT_DIR=./debug solx contract.sol
   ```
3. **Inspect stage by stage**:
   - Yul pipeline: Yul → LLVM IR (unoptimized) → LLVM IR (optimized) → assembly.
   - Legacy pipeline: EVMLA → EthIR → LLVM IR (unoptimized) → LLVM IR (optimized) → assembly.
4. **Narrow down** which stage introduces the problem.
5. **Use LLVM verification** if the issue is in the optimizer:
   ```bash
   solx contract.sol --llvm-verify-each --emit-llvm-ir -o ./debug --overwrite
   ```
6. **Compare with solc** using the integration tester:
   ```bash
   cargo run --release --bin solx-tester -- \
     --solidity-compiler ./target/release/solx \
     --path contract.sol
   ```
