//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;
    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT_PATH,
        "--libraries",
        crate::common::LIBRARY_DEFAULT,
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
        crate::common::TEST_SOLIDITY_STANDARD_JSON_PATH,
        "--libraries",
        crate::common::LIBRARY_DEFAULT,
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
        crate::common::TEST_YUL_CONTRACT_PATH,
        "--libraries",
        crate::common::LIBRARY_CONTRACT_NAME_MISSING,
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
        crate::common::TEST_YUL_CONTRACT_PATH,
        "--libraries",
        crate::common::LIBRARY_ADDRESS_MISSING,
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
        crate::common::TEST_YUL_CONTRACT_PATH,
        "--libraries",
        crate::common::LIBRARY_ADDRESS_INVALID,
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
        "tests/data/contracts/solidity/MiniMath.sol:MiniMath=0xF9702469Dfb84A9aC171E284F71615bd3D3f1EdC",
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
        "tests/data/contracts/solidity/MiniMath.sol:MiniMath=0xF9702469Dfb84A9aC171E284F71615bd3D3f1EdC",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}
