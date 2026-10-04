//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use test_case::test_case;

/// The hex-encoded `solc:<version>` entry of the CBOR metadata appended to the bytecode.
fn cbor_solc_version_hex(version: &str) -> String {
    format!("solc:{version}")
        .bytes()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn pinned_pragma() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--solidity-version",
        "0.8.20",
        "--bin",
        crate::common::contract!("solidity/PinnedPragma.sol"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains(cbor_solc_version_hex("0.8.20")));

    Ok(())
}

#[test]
fn long_version() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--solidity-version",
        "0.8.20+commit.a1b79de6",
        "--bin",
        crate::common::contract!("solidity/PinnedPragma.sol"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains(cbor_solc_version_hex("0.8.20")))
        .stderr(predicate::str::contains(
            "Solidity version 0.8.20+commit.a1b79de6 is compiled as 0.8.20",
        ));

    Ok(())
}

#[test]
fn syntax_newer_than_version() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--solidity-version",
        "0.8.26",
        "--bin",
        crate::common::contract!("solidity/TransientStorage.sol"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "This syntax was introduced in version '0.8.27'.",
    ));

    Ok(())
}

#[test]
fn default_evm_version() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--solidity-version",
        "0.8.30",
        "--emit-mlir=sol",
        crate::common::TEST_SOLIDITY_CONTRACT,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("sol.evm_version = #Prague"));

    Ok(())
}

#[test]
fn default_evm_version_unsupported() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--solidity-version",
        "0.8.24",
        "--emit-mlir=sol",
        crate::common::TEST_SOLIDITY_CONTRACT,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("sol.evm_version = #Cancun"))
        .stderr(predicate::str::contains(
            "Solidity version 0.8.24 defaults to EVM version shanghai, which solx does not support yet. Compiling for cancun",
        ));

    Ok(())
}

#[test_case("0.7.6")]
#[test_case("0.9.0")]
fn unsupported(version: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--solidity-version",
        version,
        "--bin",
        crate::common::TEST_SOLIDITY_CONTRACT,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(format!(
        "Solidity version {version} is not supported."
    )));

    Ok(())
}

#[test]
fn yul() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--solidity-version",
        "0.8.20",
        "--yul",
        "--bin",
        crate::common::TEST_YUL_CONTRACT,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "Solidity version is only allowed in Solidity mode",
    ));

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_version.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains(cbor_solc_version_hex("0.8.20")));

    Ok(())
}

#[test]
fn standard_json_unsupported() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_version_unsupported.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Solidity version 0.7.6 is not supported.",
    ));

    Ok(())
}

#[test]
fn standard_json_with_flag() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_version.json"),
        "--solidity-version",
        "0.8.20",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Solidity version must be passed via standard JSON input.",
    ));

    Ok(())
}
