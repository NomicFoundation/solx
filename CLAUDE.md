# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**solx** is an optimizing Solidity compiler for EVM developed by Matter Labs and Nomic Foundation. It uses LLVM to generate optimized EVM bytecode from Solidity source code.

For detailed architecture and compilation pipeline, see [docs/src/04-architecture.md](./docs/src/04-architecture.md).

## Crate Structure

- **solx** — Main executable and CLI argument handling
- **solx-core** — Core compiler logic: project parsing, solc integration, Yul handling, compilation orchestration
- **solx-codegen-evm** — LLVM IR to EVM bytecode generation using inkwell/llvm-sys bindings
- **solx-evm-assembly** — EVM assembly translator
- **solx-yul** — Yul lexer and parser
- **solx-standard-json** — Standard JSON protocol implementation (solc-compatible input/output)
- **solx-utils** — Shared utilities (hashing, serialization, error codes)
- **solx-dev** — Development tooling CLI for building LLVM and running project tests
- **solx-tester** — Integration testing framework with revm for EVM execution
- **solx-compiler-downloader** — Downloads external compiler versions
- **solx-benchmark-converter** — Converts benchmark results to Excel reports

### Key Entry Points

- `solx/src/solx.rs` — Main compiler executable
- `solx-core/src/lib.rs::main()` — Core compilation logic
- `solx-dev/src/solx_dev/main.rs` — Development tool entry
- `solx-tester/src/solx_tester/main.rs` — Integration tester entry

## Build Commands

**Always use `cargo run` instead of `./target/release/...`** to run binaries. This ensures binaries are rebuilt if needed.

```bash
# Run main compiler
cargo run --release --bin solx -- [args]

# Run development tool
cargo run --release --bin solx-dev -- [args]

# Run integration tester
cargo run --release --bin solx-tester -- [args]
```

### LLVM Environment Variable

If LLVM build artifacts are not found, set:
```bash
export LLVM_SYS_211_PREFIX="${HOME}/src/solx/target-llvm/build-final"
```

### Building MLIR vs Non-MLIR solx

solx can be built with or without MLIR support. The difference is determined by:
1. The `solx-solidity` submodule branch
2. The LLVM build configuration

**Non-MLIR build (standard):**
```bash
# Use the release branch for solx-solidity
cd solx-solidity && git checkout origin/0.8.33 && cd ..

# Build LLVM without MLIR
cargo run --release --bin solx-dev -- llvm --clone --build

# Build solx
cargo build --release --bin solx
```

**MLIR build:**
```bash
# Use the main branch for solx-solidity (includes MLIR support)
cd solx-solidity && git checkout origin/main && cd ..

# Build LLVM with MLIR support
cargo run --release --bin solx-dev -- llvm --clone --build --llvm-projects mlir lld --enable-rtti

# Build solx
cargo build --release --bin solx
```

The resulting `solx` binary will include MLIR support or not based on the submodule version.
You can verify which version you have by checking the version output:
- MLIR: `Version: 0.8.33-develop...`
- Non-MLIR: `Version: 0.8.33+commit...`

## Testing

For detailed test format documentation, see [docs/src/05-testing.md](./docs/src/05-testing.md).

```bash
# Run all tests (unit + CLI)
cargo test

# Run only unit tests
cargo test --lib

# Run only CLI tests
cargo test --test cli

# Run integration tests with solx-tester (auto-detects solx vs solc)
cargo run --release --bin solx-tester -- --solidity-compiler ./target/release/solx

# Run integration tests on specific path
cargo run --release --bin solx-tester -- --solidity-compiler ./target/release/solx --path tests/solidity/simple/default.sol

# Run with upstream solc (if installed)
cargo run --release --bin solx-tester -- --solidity-compiler $(which solc) --path tests/solidity/simple/default.sol

# Run Foundry project tests
cargo run --release --bin solx-dev -- test foundry --test-config-path solx-dev/foundry-tests.toml

# Run Hardhat project tests
cargo run --release --bin solx-dev -- test hardhat --test-config-path solx-dev/hardhat-tests.toml
```

### Test Data Locations

- `solx/tests/data/` — Unit test data (contracts, standard JSON inputs)
- `tests/solidity/` — Solidity integration tests
- `tests/yul/` — Yul integration tests
- `tests/llvm-ir/` — LLVM IR integration tests

## Code Style

- **Imports**: One item per line (no grouped imports like `use foo::{A, B}`), for easy `dd` deletion in vim. Order: `std`, external crates, workspace crates, `crate`, `self` (separated by an empty line)
- **Variable names**: No contractions — use `error` not `e`, `address` not `addr`, `transaction` not `tx`
- **References**: Prefer `.as_ref()` over `&` for Option/Result types
- **Control flow**: Reduce nesting with `let ... else { continue }` pattern in loops instead of nested `if let`
- **Function ordering**: In test modules, place test functions (`#[test] fn ...`) above helper/private functions
- **Before committing**: Always run `cargo fmt` and `cargo clippy` before every commit

## Configuration Files

- `rust-toolchain.toml` — Pinned Rust version
- `solx-dev/foundry-tests.toml` — Foundry project test configuration
- `solx-dev/hardhat-tests.toml` — Hardhat project test configuration
- `solx-compiler-downloader/*.json` — Compiler version references

## Documentation

Documentation is in `docs/` as an mdBook project:
```bash
cd docs
mdbook serve  # Serve locally at http://localhost:3000
mdbook build  # Build static HTML
```
