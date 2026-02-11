//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--base-path",
        "tests/data/contracts/",
        "--include-path",
        "tests/data/contracts/",
        "--bin",
        crate::common::contract!("solidity/Test.sol"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Binary:\n"));

    Ok(())
}

#[test]
fn yul() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--base-path",
        "tests/data/contracts/",
        "--include-path",
        "tests/data/contracts/",
        "--yul",
        "--bin",
        crate::common::contract!("yul/Test.yul"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "`include-path` is only allowed in Solidity mode",
    ));

    Ok(())
}

#[test]
fn llvm_ir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--base-path",
        "tests/data/contracts/",
        "--include-path",
        "tests/data/contracts/",
        "--llvm-ir",
        "--bin",
        crate::common::contract!("llvm_ir/Test.ll"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "`include-path` is only allowed in Solidity mode",
    ));

    Ok(())
}

#[test]
fn base_path_missing() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--include-path",
        "tests/data/contracts/",
        "--bin",
        crate::common::contract!("solidity/Test.sol"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "--include-path option requires a non-empty base path",
    ));

    Ok(())
}
