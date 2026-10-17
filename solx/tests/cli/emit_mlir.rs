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

    let sol_deploy_index = stdout
        .find("MLIR Dialect sol (deploy):")
        .expect("sol deploy dialect header missing");
    let sol_runtime_index = stdout
        .find("MLIR Dialect sol (runtime):")
        .expect("sol runtime dialect header missing");
    let deploy_index = stdout
        .find("MLIR Dialect yul (deploy):")
        .expect("yul deploy dialect header missing");
    let runtime_index = stdout
        .find("MLIR Dialect yul (runtime):")
        .expect("yul runtime dialect header missing");
    assert!(
        sol_deploy_index < sol_runtime_index
            && sol_runtime_index < deploy_index
            && deploy_index < runtime_index,
        "expected sol stages before yul stages in pipeline order, each deploy before runtime"
    );
    assert!(stdout.contains("sol.contract"));
    assert!(stdout.contains("yul.func"));
    assert!(!stdout.contains(" loc(") && !stdout.contains("#loc"));

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
        .stdout(predicate::str::contains("MLIR Dialect sol (deploy):"))
        .stdout(predicate::str::contains("MLIR Dialect sol (runtime):"))
        .stdout(predicate::str::contains("sol.contract"))
        .stdout(predicate::str::contains("MLIR Dialect yul").not());

    Ok(())
}

#[test]
fn filter_yul_only() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--emit-mlir=yul",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("MLIR Dialect yul (deploy):"))
        .stdout(predicate::str::contains("MLIR Dialect yul (runtime):"))
        .stdout(predicate::str::contains("yul.func"))
        .stdout(predicate::str::contains("MLIR Dialect sol").not());

    Ok(())
}

#[test]
fn locations_with_runtime_debug_info() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--emit-mlir=sol",
        "--debug-info-runtime",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("sol.contract"))
        .stdout(predicate::str::contains(" loc("));

    Ok(())
}

#[test]
fn runtime_location_aliases() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--emit-mlir=yul",
        "--debug-info-runtime",
    ];

    let result = crate::cli::execute_solx(args)?;
    let stdout = String::from_utf8(result.success().get_output().stdout.clone())?;

    let runtime_index = stdout
        .find("MLIR Dialect yul (runtime):")
        .expect("yul runtime dialect header missing");
    assert!(
        stdout[runtime_index..].contains("\n#loc"),
        "the runtime module is printed with its locations as `#loc` aliases"
    );

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
