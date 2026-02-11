//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--llvm-options='-evm-metadata-size 10'",
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
        "--llvm-options='-evm-metadata-size 10'",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "LLVM options must be specified in standard JSON input settings.",
    ));

    Ok(())
}
