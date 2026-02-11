//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;

#[test]
fn missing_input() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &["--recursive-process"];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "Input length prefix reading error: failed to fill whole buffer",
    ));

    Ok(())
}

#[test]
fn excess_args() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--recursive-process",
        crate::common::contract!("solidity/Test.sol"),
        "excess",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "No other options are allowed while running in the recursive process mode.",
    ));

    Ok(())
}
