//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;
    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--libraries",
        "tests/data/contracts/solidity/MiniMath.sol:MiniMath=0xF9702469Dfb84A9aC171E284F71615bd3D3f1EdC",
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
        crate::common::standard_json!("solidity.json"),
        "--libraries",
        "tests/data/contracts/solidity/MiniMath.sol:MiniMath=0xF9702469Dfb84A9aC171E284F71615bd3D3f1EdC",
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
        crate::common::contract!("yul/Test.yul"),
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
        crate::common::contract!("yul/Test.yul"),
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
        crate::common::contract!("yul/Test.yul"),
        "--libraries",
        "tests/data/contracts/solidity/MiniMath.sol:MiniMath=INVALID",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "Error: Invalid address `INVALID` of library `tests/data/contracts/solidity/MiniMath.sol:MiniMath`: Odd number of digits",
    ));

    Ok(())
}
