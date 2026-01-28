//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use test_case::test_case;

#[test]
fn bin() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_YUL_CONTRACT_PATH, "--yul", "--bin"];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

#[test]
fn stdin() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--yul",
        "--bin",
        solx_standard_json::InputSource::STDIN_INPUT_IDENTIFIER,
    ];

    let result = crate::cli::execute_solx_with_stdin(args, crate::common::TEST_YUL_CONTRACT_PATH)?;

    result
        .success()
        .stdout(predicate::str::contains("Binary").count(1));

    Ok(())
}

#[test]
fn asm() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_YUL_CONTRACT_PATH, "--yul", "--asm"];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("assembly"));

    Ok(())
}

#[test]
fn metadata() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_YUL_CONTRACT_PATH, "--yul", "--metadata"];

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

    let args = &[crate::common::TEST_YUL_CONTRACT_PATH, "--yul", flag];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "can be only emitted for Solidity contracts",
    ));

    Ok(())
}

#[test]
fn object_naming() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_YUL_CONTRACT_OBJECT_NAMING_PATH,
        "--yul",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

#[test]
fn solc() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_YUL_CONTRACT_PATH, "--yul", "--bin"];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

#[test]
fn invalid_input() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT_PATH, "--yul"];

    let result = crate::cli::execute_solx(args)?;
    result
        .failure()
        .stderr(predicate::str::contains("Yul parsing"));

    Ok(())
}

#[test]
fn invalid_standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_YUL_CONTRACT_PATH,
        "--yul",
        "--standard-json",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Only one mode is allowed at the same time:",
    ));

    Ok(())
}

#[test]
fn standard_json_default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_YUL_STANDARD_JSON_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"))
        .stdout(predicate::str::contains("object"));

    Ok(())
}

#[test]
fn standard_json_default_urls() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_YUL_STANDARD_JSON_URLS_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"))
        .stdout(predicate::str::contains("object"));

    Ok(())
}

#[test]
fn standard_json_default_urls_invalid() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_YUL_STANDARD_JSON_URLS_INVALID_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "DeclarationError: Function \\\"mdelete\\\" not found.",
    ));

    Ok(())
}

#[test]
fn standard_json_default_urls_debug_info() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_YUL_STANDARD_JSON_URLS_DEBUG_INFO_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Error: Debug info is only supported for Solidity source code input.",
    ));

    Ok(())
}
