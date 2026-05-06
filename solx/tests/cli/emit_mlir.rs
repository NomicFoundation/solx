//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--emit-mlir",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    let stdout = String::from_utf8(result.success().get_output().stdout.clone())?;

    let sol_index = stdout
        .find("MLIR Dialect sol:")
        .expect("sol dialect header missing");
    let deploy_index = stdout
        .find("MLIR Dialect llvm (deploy):")
        .expect("llvm deploy dialect header missing");
    let runtime_index = stdout
        .find("MLIR Dialect llvm (runtime):")
        .expect("llvm runtime dialect header missing");
    assert!(
        sol_index < deploy_index && deploy_index < runtime_index,
        "expected sol stage before llvm deploy and runtime stages in pipeline order"
    );
    assert!(stdout.contains("sol.contract"));
    assert!(stdout.contains("llvm.func"));

    Ok(())
}

#[test]
fn filter_sol_only() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--emit-mlir=sol",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("MLIR Dialect sol:"))
        .stdout(predicate::str::contains("sol.contract"))
        .stdout(predicate::str::contains("MLIR Dialect llvm").not());

    Ok(())
}

#[test]
fn filter_llvm_only() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--emit-mlir=llvm",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("MLIR Dialect llvm (deploy):"))
        .stdout(predicate::str::contains("MLIR Dialect llvm (runtime):"))
        .stdout(predicate::str::contains("llvm.func"))
        .stdout(predicate::str::contains("MLIR Dialect sol:").not());

    Ok(())
}

#[test]
fn rejects_invalid_dialect() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--emit-mlir=foo",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .failure()
        .stderr(predicate::str::contains("invalid value 'foo'"));

    Ok(())
}

#[test]
fn standard_json_error() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON,
        "--emit-mlir",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Cannot output data outside of JSON in standard JSON mode.",
    ));

    Ok(())
}
