//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::contract!("solidity/Test.sol"), "--asm"];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("Deploy LLVM EVM assembly").count(1))
        .stdout(predicate::str::contains("Runtime LLVM EVM assembly").count(1));

    Ok(())
}

#[test]
fn invalid_input() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::contract!("yul/Test.yul"), "--asm"];

    let result = crate::cli::execute_solx(args)?;

    result.failure().stderr(predicate::str::contains(
        "Expected identifier but got 'StringLiteral'",
    ));

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity.json"),
        "--asm",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Cannot output data outside of JSON in standard JSON mode.",
    ));

    Ok(())
}
