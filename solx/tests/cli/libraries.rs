//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;

/// Library linking argument for the MiniMath contract at a fixed address.
/// Format: `<source-path>:<contract-name>=<address>`
const LIBRARY_MINIMATH: &str = "tests/data/contracts/solidity/MiniMath.sol:MiniMath=0xF9702469Dfb84A9aC171E284F71615bd3D3f1EdC";

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;
    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--libraries",
        LIBRARY_MINIMATH,
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON,
        "--libraries",
        LIBRARY_MINIMATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Libraries must be passed via standard JSON input.",
    ));

    Ok(())
}

#[test]
fn missing_contract_name() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--yul",
        crate::common::TEST_YUL_CONTRACT,
        "--libraries",
        "tests/data/contracts/solidity/MiniMath.sol=0xF9702469Dfb84A9aC171E284F71615bd3D3f1EdC",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "Library `tests/data/contracts/solidity/MiniMath.sol` contract name is missing.",
    ));

    Ok(())
}

#[test]
fn missing_address() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--yul",
        crate::common::TEST_YUL_CONTRACT,
        "--libraries",
        "tests/data/contracts/solidity/MiniMath.sol:MiniMath",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "Error: Library `tests/data/contracts/solidity/MiniMath.sol:MiniMath` address is missing.",
    ));

    Ok(())
}

#[test]
fn invalid_address() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--yul",
        crate::common::TEST_YUL_CONTRACT,
        "--libraries",
        "tests/data/contracts/solidity/MiniMath.sol:MiniMath=INVALID",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "Error: Invalid address `INVALID` of library `tests/data/contracts/solidity/MiniMath.sol:MiniMath`: Odd number of digits",
    ));

    Ok(())
}

#[test]
fn linked_mixed_deps() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/LinkedMixedDeps.sol"),
        "--bin",
        "--libraries",
        LIBRARY_MINIMATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

#[test]
fn linked_mixed_deps_multi_level() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/LinkedMixedDepsMultiLevel.sol"),
        "--bin",
        "--libraries",
        LIBRARY_MINIMATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}
