//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--via-ir",
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
        "--yul",
        "--via-ir",
        "--bin",
        crate::common::contract!("yul/Test.yul"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "Error: IR codegen settings are only available in Solidity mode.",
    ));

    Ok(())
}

#[test]
fn llvm_ir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--llvm-ir",
        "--via-ir",
        "--bin",
        crate::common::contract!("llvm_ir/Test.ll"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "Error: IR codegen settings are only available in Solidity mode.",
    ));

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--via-ir",
        "--standard-json",
        crate::common::standard_json!("solidity.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "IR codegen must be passed via standard JSON input.",
    ));

    Ok(())
}

#[test]
fn emit_llvm_ir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--via-ir",
        "--emit-llvm-ir",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Deploy LLVM IR:"))
        .stdout(predicate::str::contains("Runtime LLVM IR:"))
        .stdout(predicate::str::contains("target datalayout"));

    Ok(())
}
