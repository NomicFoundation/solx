# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**solx** is an optimizing Solidity compiler for EVM, developed by Matter Labs and Nomic Foundation. It uses a custom LLVM backend with an EVM target to generate optimized bytecode. The project is in beta.

## Architecture

Three-repository structure via git submodules:
- **solx** (this repo) — Rust workspace: CLI, compilation pipeline, LLVM IR generation
- **solx-solidity** (submodule) — Fork of solc that emits Yul/EVM assembly
- **solx-llvm** (submodule) — Fork of LLVM with an EVM target backend

**Compilation pipeline:**
```
Solidity → solc frontend (parse, analyze) → Yul/EVM assembly → LLVM IR → LLVM optimizer → EVM bytecode
```

Alternative pipeline via Slang (Rust-based frontend, WIP):
```
Solidity → Slang (parse, analyze) → MLIR → LLVM IR → EVM bytecode
```

### Workspace Crates (14 total)

| Crate | Purpose |
|---|---|
| `solx` | CLI entry point, frontend dispatch |
| `solx-core` | Pipeline orchestration (`standard_output_evm`, `standard_json_evm`, `yul_to_evm`, `llvm_ir_to_evm`) |
| `solx-codegen-evm` | LLVM IR generation, optimization passes, bytecode assembly, linking |
| `solx-slang` | Alternative Rust-based Slang frontend |
| `solx-mlir` | MLIR-to-LLVM translation via melior |
| `solx-yul` | Yul lexer/parser |
| `solx-evm-assembly` | EVM assembly representation and ethereal IR |
| `solx-standard-json` | solc-compatible JSON I/O protocol |
| `solx-utils` | Shared types (contract names, EVM versions, hashing, CBOR metadata) |
| `solx-dev` | Developer tool: builds LLVM, solc, runs project tests |
| `solx-tester` | Integration test runner using REVM |
| `solx-compiler-downloader` | Downloads/verifies compiler binaries |
| `solx-benchmark-converter` | Benchmark analysis and comparison |
| `solx-solc-test-adapter` | Adapter for upstream solc tests |

**Important feature split:** `solx-slang` and `solx-mlir` use inkwell with `llvm21-1-no-llvm-linking`, while `solx-codegen-evm` uses `llvm21-1` (links LLVM). They are excluded from `default-members` to avoid Cargo feature unification conflicts. Use `cargo test-slang` to test them.

## Build Commands

### Prerequisites

Build the `solx-dev` tool first, then use it to build LLVM and solc:

```bash
cargo build --release --bin solx-dev

# Build LLVM (outputs to target-llvm/target-final/)
./target/release/solx-dev llvm build --enable-mlir

# Build solc libraries (outputs to solx-solidity/build/)
./target/release/solx-dev solc build
```

### Building solx

```bash
cargo build              # debug
cargo build --release    # release (outputs to target/release/solx)
```

### Environment Variables (auto-set via .cargo/config.toml)

- `LLVM_SYS_211_PREFIX` → `./target-llvm/target-final/`
- `MLIR_SYS_210_PREFIX` → `./target-llvm/target-final/`
- `TABLEGEN_210_PREFIX` → `./target-llvm/target-final/`
- `SOLC_PREFIX` → `./solx-solidity/build/`
- `BOOST_PREFIX` → `./solx-solidity/boost/lib/`

## Testing

### Unit and CLI tests

```bash
cargo test                              # all tests (unit + CLI)
cargo test --lib                        # unit tests only
cargo test --test cli                   # CLI integration tests only
cargo test --test cli -- cli::bin::default  # specific test
```

### Slang/MLIR tests (cargo alias)

```bash
cargo test-slang
# expands to: cargo test -p solx-slang -p solx-mlir -p solx --no-default-features --features slang --target-dir target-slang
```

### Integration tests with solx-tester

```bash
cargo build --release
./target/release/solx-tester --solidity-compiler ./target/release/solx
./target/release/solx-tester --solidity-compiler ./target/release/solx --path tests/solidity/simple/default.sol
./target/release/solx-tester --solidity-compiler ./target/release/solx --via-ir  # Yul pipeline only
```

### Project tests (Foundry/Hardhat)

```bash
./target/release/solx-dev test foundry --test-config-path solx-dev/foundry-tests.toml
./target/release/solx-dev test hardhat --test-config-path solx-dev/hardhat-tests.toml
```

## Code Conventions

- Rust edition 2024, toolchain 1.93.0
- `missing_docs` warning is enabled globally — all public items need doc comments
- Release builds strip debug symbols (`strip = true`)
- macOS minimum deployment target: 11.0

## Test File Format

Tests live in `tests/solidity/`, `tests/yul/`, `tests/llvm-ir/`.

- **Simple tests** (`tests/solidity/simple/`): single file with `//!` JSON metadata comments
- **Complex tests** (`tests/solidity/complex/`): directory with `test.json` + source files
- Test metadata includes: `cases`, `contracts`, `libraries`, `ignore`, `modes` (`Y` for Yul, `E` for EVMLA)

## CI Labels

- `ci:slang` — enable Slang tests
- `ci:sanitizer` — enable address sanitizer tests
- `ci:integration` — enable integration tests
- `ci:coverage` — enable code coverage
