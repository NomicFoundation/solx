//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use test_case::test_case;

#[test_case(solx_utils::EVMVersion::Cancun)]
#[test_case(solx_utils::EVMVersion::Prague)]
#[test_case(solx_utils::EVMVersion::Osaka)]
fn default(evm_version: solx_utils::EVMVersion) -> anyhow::Result<()> {
    crate::common::setup()?;

    let evm_version = evm_version.to_string();
    let args = &[
        "--evm-version",
        evm_version.as_str(),
        "--bin",
        crate::common::TEST_SOLIDITY_CONTRACT_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Binary:\n"));

    Ok(())
}

#[test_case(solx_utils::EVMVersion::Cancun)]
#[test_case(solx_utils::EVMVersion::Prague)]
#[test_case(solx_utils::EVMVersion::Osaka)]
fn yul(evm_version: solx_utils::EVMVersion) -> anyhow::Result<()> {
    crate::common::setup()?;

    let evm_version = evm_version.to_string();
    let args = &[
        "--evm-version",
        evm_version.as_str(),
        "--yul",
        "--bin",
        crate::common::TEST_YUL_CONTRACT_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "EVM version is only allowed in Solidity mode",
    ));

    Ok(())
}

#[test_case(solx_utils::EVMVersion::Cancun)]
#[test_case(solx_utils::EVMVersion::Prague)]
#[test_case(solx_utils::EVMVersion::Osaka)]
fn llvm_ir(evm_version: solx_utils::EVMVersion) -> anyhow::Result<()> {
    crate::common::setup()?;

    let evm_version = evm_version.to_string();
    let args = &[
        "--evm-version",
        evm_version.as_str(),
        "--llvm-ir",
        "--bin",
        crate::common::TEST_LLVM_IR_CONTRACT_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "EVM version is only allowed in Solidity mode",
    ));

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let evm_version = solx_utils::EVMVersion::Cancun.to_string();
    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON_PATH,
        "--evm-version",
        evm_version.as_str(),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "EVM version must be passed via standard JSON input.",
    ));

    Ok(())
}

#[test]
fn too_old() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--evm-version",
        "shanghai",
        "--bin",
        crate::common::TEST_SOLIDITY_CONTRACT_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .failure()
        .stderr(predicate::str::contains("Unsuppored EVM version"));

    Ok(())
}

#[test]
fn standard_json_too_old() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_JSON_EVM_VERSION_TOO_OLD,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Standard JSON parsing: unknown variant",
    ));

    Ok(())
}
