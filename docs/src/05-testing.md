# Testing

This page describes how to run tests for the **solx** compiler and the format of test files.

## Unit and CLI Tests

Run the standard Rust test suite:

```shell
# Run all tests (unit + CLI)
cargo test

# Run only unit tests
cargo test --lib

# Run only CLI/integration tests
cargo test --test cli

# Run a specific test
cargo test --test cli -- cli::bin::default
```

## Integration Tests

The **solx-tester** tool runs integration tests by compiling contracts and executing them with [revm](https://github.com/bluealloy/revm).

```shell
# Build the compiler and tester
cargo build --release

# Run all integration tests
./target/release/solx-tester --solx ./target/release/solx

# Run tests for a specific file
./target/release/solx-tester --solx ./target/release/solx --path tests/solidity/simple/default.sol

# Run tests matching a mode
./target/release/solx-tester --solx ./target/release/solx --mode "Y+M3B3 0.8.33"
```

## Foundry and Hardhat Projects

The **solx-dev** tool can run tests against real-world Foundry and Hardhat projects:

```shell
# Build solx-dev
cargo build --release --bin solx-dev

# Run Foundry project tests
./target/release/solx-dev test foundry --test-config-path solx-dev/foundry-tests.toml

# Run Hardhat project tests
./target/release/solx-dev test hardhat --test-config-path solx-dev/hardhat-tests.toml
```

The test configurations list projects that are cloned and tested automatically. See [foundry-tests.toml](https://github.com/NomicFoundation/solx/blob/main/solx-dev/foundry-tests.toml) and [hardhat-tests.toml](https://github.com/NomicFoundation/solx/blob/main/solx-dev/hardhat-tests.toml) for the full list of tested projects.

## Test Collection

This section describes the format of test files used by **solx-tester**.

### Test Types

The repository contains three types of tests:

- **Upstream** — Tests following the [Solidity semantic test format](https://github.com/NomicFoundation/solx-solidity/tree/0.8.33/test/libsolidity/semanticTests).
- **Simple** — Single-contract tests.
- **Complex** — Multi-contract tests and vendored DeFi projects.

Test data is located in:
- `tests/solidity/` — Solidity test contracts
- `tests/yul/` — Yul test contracts
- `tests/llvm-ir/` — LLVM IR test contracts

### Test Format

Each test comprises source code files and metadata.
Simple tests have only one source file, and their metadata is written in comments that start with `!`, for example, `//!` for Solidity.
Complex tests use a `test.json` file to describe their metadata and refer to source code files.

### Metadata

Metadata is a JSON object that contains the following fields:

- `cases` — An array of test cases (described below).
- `contracts` — Used for complex tests to describe the contract instances to deploy. In simple tests, only one `Test` contract instance is deployed.
```json
"contracts": {
    "Main": "main.sol:Main",
    "Callable": "callable.sol:Callable"
}
```
- `libraries` — An optional field that specifies library addresses for linker:
```json
"libraries": {
    "libraries/UQ112x112.sol": { "UQ112x112": "UQ112x112" },
    "libraries/Math.sol": { "Math": "Math" }
}
```
- `ignore` — An optional flag that disables a test.
- `modes` — An optional field that specifies mode filters. `Y` stands for Yul pipeline, `E` for EVM assembly pipeline. Compiler versions can be specified as SemVer ranges:
```json
"modes": [
    "Y+",
    "E+",
    "E+ >=0.8.30"
]
```
- `group` — An optional string field that specifies a test group for benchmarking.

### Test Cases

All test cases are executed in a clean context, making them independent of each other.

Each test case contains the following fields:

- `name` — A string name.
- `comment` — An optional string comment.
- `inputs` — An array of inputs (described below).
- `expected` — The expected return data for the last input.
- `ignore`, `modes` — Same as in test metadata.

### Inputs

Inputs specify the contract calls in the test case:

- `comment` — An optional string comment.
- `instance` — The contract instance to call. Default: `Test`.
- `caller` — The caller address. Default: `0xdeadbeef01000000000000000000000000000000`.
- `method` — The method to call:
    1. `#deployer` for the deployer call.
    2. `#fallback` to perform a call with raw calldata.
    3. Any other string is recognized as a function name. The function selector will be prepended to the calldata.
- `calldata` — The input calldata:
    1. A hexadecimal string: `"calldata": "0x00"`
    2. A numbers array (hex, decimal, or instance addresses). Each number is padded to 32 bytes: `"calldata": ["1", "2"]`
- `value` — An optional `msg.value`, a decimal number with ` wei` or ` ETH` suffix.
- `storage` — Storage values to set before the call:
```json
"storage": {
    "Test.address": ["1", "2", "3", "4"]
}
```
- `expected` — The expected return data:
    1. An array of numbers: `"expected": ["1", "2"]`
    2. Extended format with `return_data`, `exception`, and `events`:
```json
"expected": {
    "return_data": ["0x01"],
    "events": [
        {
            "topics": [
                "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
            ],
            "values": ["0xff"]
        }
    ],
    "exception": false
}
```

The `expected` field can be an array of objects if different expected data is needed for different compiler versions. Use `compiler_version` as a SemVer range in extended expected format.

Notes:
- `InstanceName.address` can be used in expected, calldata, and storage fields to insert a contract instance address.
- If a deployer call is not specified for an instance, it will be generated automatically with empty calldata.

### Upstream Solidity Semantic Tests

These tests follow the [Solidity semantic test format](https://github.com/NomicFoundation/solx-solidity/tree/0.8.33/test/libsolidity/semanticTests).
Test descriptions and expected results are embedded as comments in the test file. Lines begin with `//` for Solidity files. The beginning of the test description is indicated by a comment line containing `----`.
