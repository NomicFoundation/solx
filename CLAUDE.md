# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**solx** is an optimizing Solidity compiler for EVM developed by Matter Labs and Nomic Foundation. It uses LLVM to generate optimized EVM bytecode from Solidity source code.

## Architecture

The compiler consists of three main parts:

1. **solx** (this repository) - The main compiler executable and Rust crates for frontend processing (Yul/EVM assembly translation)
2. **solx-solidity** (submodule) - LLVM-friendly fork of the Solidity compiler that emits Yul and EVM assembly
3. **solx-llvm** (submodule) - Fork of LLVM with an EVM target backend

### Crate Structure

- **solx** - Main executable and CLI argument handling
- **solx-core** - Core compiler logic: project parsing, solc integration, Yul handling, compilation orchestration
- **solx-codegen-evm** - LLVM IR to EVM bytecode generation using inkwell/llvm-sys bindings
- **solx-evm-assembly** - EVM assembly translator
- **solx-yul** - Yul lexer and parser
- **solx-standard-json** - Standard JSON protocol implementation (solc-compatible input/output)
- **solx-utils** - Shared utilities (hashing, serialization, error codes)
- **solx-dev** - Development tooling CLI (`solx-dev`) for building LLVM and running project tests
- **solx-tester** - Integration testing framework (`solx-tester`) with revm for EVM execution
- **solx-compiler-downloader** - Downloads external compiler versions
- **solx-benchmark-converter** - Converts benchmark results to Excel reports

### Key Entry Points

- `solx/src/solx.rs` - Main compiler executable
- `solx-core/src/lib.rs::main()` - Core compilation logic
- `solx-dev/src/solx_dev/main.rs` - Development tool entry
- `solx-tester/src/solx_tester/main.rs` - Integration tester entry

## Build Commands

```bash
# Build main compiler (release)
cargo build --release --bin solx

# Build all binaries
cargo build --release

# Build development tool
cargo build --release --bin solx-dev

# Build integration tester
cargo build --release --bin solx-tester
```

### Building from Source (Full Setup)

1. Install dependencies (cmake, ninja, clang, lld)
2. Clone with submodules: `git clone --recursive` or `git submodule update --recursive --checkout`
3. Build LLVM: `solx-llvm build` (or use `cargo install compiler-llvm-builder` first)
4. Build solc libraries in `solx-solidity/build/` using cmake
5. Build solx: `cargo build --release`

### LLVM Environment Variable

If LLVM build artifacts are not found, set:
```bash
export LLVM_SYS_211_PREFIX="${HOME}/src/solx/target-llvm/build-final"
```

## Testing

```bash
# Run all tests (unit + CLI)
cargo test

# Run only unit tests
cargo test --lib

# Run only CLI/e2e tests
cargo test --test test_cli

# Run a specific test
cargo test --test test_cli -- cli::bin::default

# Run integration tests with solx-tester
./target/release/solx-tester --solx ./target/release/solx

# Run integration tests on specific path
./target/release/solx-tester --solx ./target/release/solx --path tests/solidity/simple/default.sol

# Run Foundry project tests
./target/release/solx-dev test foundry --test-config-path solx-dev/foundry-tests.toml

# Run Hardhat project tests
./target/release/solx-dev test hardhat --test-config-path solx-dev/hardhat-tests.toml
```

Test data is located in `solx/tests/data/` with contracts in `tests/data/contracts/` and standard JSON inputs in `tests/data/standard_json_input/`.

## solx-dev Commands

```bash
# Build LLVM (from repo root)
./target/release/solx-dev llvm build

# Build LLVM with options
./target/release/solx-dev llvm build --build-type Release --enable-assertions --ccache-variant ccache

# Run Foundry tests
./target/release/solx-dev test foundry --test-config-path solx-dev/foundry-tests.toml

# Run Hardhat tests
./target/release/solx-dev test hardhat --test-config-path solx-dev/hardhat-tests.toml
```

## Compilation Pipeline

1. **Solidity input** → solc (via solx-solidity) produces Yul IR or EVM assembly
2. **IR Analysis** → solx-core parses and validates the intermediate representation
3. **LLVM Compilation** → solx-codegen-evm generates LLVM IR, optimizes, and produces EVM bytecode
4. **Linking** → Deploy-time library linking if needed

The compiler supports three input languages:
- Solidity (via standard JSON or direct paths)
- Yul (standalone Yul files)
- LLVM IR (for testing/debugging)

## Configuration Files

- `rust-toolchain.toml` - Pinned Rust version (1.93.0)
- `solx-dev/foundry-tests.toml` - Foundry project test configuration
- `solx-dev/hardhat-tests.toml` - Hardhat project test configuration
- `solx-compiler-downloader/solc-bin-*.json` - Compiler version references

## Code Style

- **Imports**: One item per line (no grouped imports like `use foo::{A, B}`), for easy `dd` deletion in vim
- **Variable names**: No contractions - use `error` not `e`, `address` not `addr`, `transaction` not `tx`
- **References**: Prefer `.as_ref()` over `&` for Option/Result types
- **Control flow**: Reduce nesting with `let ... else { continue }` pattern in loops instead of nested `if let`
- **Function ordering**: In test modules, place test functions (`#[test] fn ...`) above helper/private functions
- **Before committing**: Always run `cargo fmt` and `cargo clippy` before every commit

## Documentation

Documentation is in `docs/` as an mdBook project:
```bash
cd docs
mdbook serve  # Serve locally at http://localhost:3000
mdbook build  # Build static HTML
```
