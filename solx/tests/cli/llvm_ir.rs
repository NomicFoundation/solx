//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use test_case::test_case;

#[test]
fn bin() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_LLVM_IR_CONTRACT_PATH,
        "--llvm-ir",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

#[test]
fn stdin() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--llvm-ir",
        "--bin",
        solx_standard_json::InputSource::STDIN_INPUT_IDENTIFIER,
    ];

    let result =
        crate::cli::execute_solx_with_stdin(args, crate::common::TEST_LLVM_IR_CONTRACT_PATH)?;

    result
        .success()
        .stdout(predicate::str::contains("Binary").count(1));

    Ok(())
}

#[test]
fn asm() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_LLVM_IR_CONTRACT_PATH,
        "--llvm-ir",
        "--asm",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("assembly"));

    Ok(())
}

#[test]
fn metadata() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_LLVM_IR_CONTRACT_PATH,
        "--llvm-ir",
        "--metadata",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Metadata"));

    Ok(())
}

#[test_case("--ast-json")]
#[test_case("--abi")]
#[test_case("--hashes")]
#[test_case("--userdoc")]
#[test_case("--devdoc")]
#[test_case("--storage-layout")]
#[test_case("--transient-storage-layout")]
#[test_case("--asm-solc-json")]
#[test_case("--ir")]
fn unavailable(flag: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_LLVM_IR_CONTRACT_PATH, "--llvm-ir", flag];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "can be only emitted for Solidity contracts",
    ));

    Ok(())
}

#[test]
fn invalid_input_text() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &["--llvm-ir", "--bin", crate::common::TEST_BROKEN_INPUT_PATH];

    let result = crate::cli::execute_solx(args)?;
    result
        .failure()
        .stderr(predicate::str::contains("error: expected top-level entity"));

    Ok(())
}

#[test]
fn invalid_input_solidity() -> anyhow::Result<()> {
    crate::common::setup()?;
    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT_PATH,
        "--llvm-ir",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .failure()
        .stderr(predicate::str::contains("error: expected top-level entity"));

    Ok(())
}

#[test]
fn invalid_input_llvm_ir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--llvm-ir",
        "--bin",
        crate::common::TEST_LLVM_IR_CONTRACT_INVALID_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "error: use of undefined value \'%runtime\'",
    ));

    Ok(())
}

#[test]
fn missing_file() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &["--llvm-ir", "--bin", crate::common::TEST_NON_EXISTENT_PATH];

    let result = crate::cli::execute_solx(args)?;
    result
        .failure()
        .stderr(predicate::str::contains("reading:"));

    Ok(())
}

#[test]
fn linker_error() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--llvm-ir",
        "--bin",
        crate::common::TEST_LLVM_IR_CONTRACT_LINKER_ERROR_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "LLVM ERROR: cannot evaluate undefined symbol 'foo'",
    ));

    Ok(())
}

#[test]
fn excess_mode_standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_LLVM_IR_CONTRACT_PATH,
        "--llvm-ir",
        "--standard-json",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Only one mode is allowed at the same time",
    ));

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_LLVM_IR_STANDARD_JSON_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"bytecode\""))
        .stdout(predicate::str::contains("\"object\""));

    Ok(())
}

#[test]
fn standard_json_invalid() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_LLVM_IR_STANDARD_JSON_INVALID_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "error: use of undefined value '%runtime'",
    ));

    Ok(())
}

#[test]
fn standard_json_missing_file() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_LLVM_IR_STANDARD_JSON_MISSING_FILE_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Error: File \\\"tests/data/contracts/llvm_ir/Missing.ll\\\" reading:",
    ));

    Ok(())
}

#[test]
fn standard_json_debug_info() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_LLVM_IR_STANDARD_JSON_DEBUG_INFO_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Error: Debug info is only supported for Solidity source code input.",
    ));

    Ok(())
}
