//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--emit-llvm-ir",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("Deploy LLVM IR (unoptimized):"))
        .stdout(predicate::str::contains("Deploy LLVM IR:"))
        .stdout(predicate::str::contains("target datalayout"));

    Ok(())
}

#[test]
fn marks_only_the_runtime_module_with_the_segment_flag() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--emit-llvm-ir",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    let stdout = String::from_utf8(result.success().get_output().stdout.clone())?;

    assert_eq!(stdout.matches("!evm.runtime = !{}").count(), 2);
    for (position, _) in stdout.match_indices("!evm.runtime = !{}") {
        let deploy_header = stdout[..position].rfind("Deploy LLVM IR").unwrap_or(0);
        let runtime_header = stdout[..position]
            .rfind("Runtime LLVM IR")
            .expect("the flag appears under a runtime section");
        assert!(
            runtime_header > deploy_header,
            "the flag lies in a runtime section"
        );
    }

    Ok(())
}

#[test]
fn llvm_ir_input() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_LLVM_IR_CONTRACT,
        "--llvm-ir",
        "--emit-llvm-ir",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("LLVM IR:"))
        .stdout(predicate::str::contains("target datalayout"));

    Ok(())
}

#[test]
fn standard_json_error() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON,
        "--emit-llvm-ir",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Cannot output data outside of JSON in standard JSON mode.",
    ));

    Ok(())
}
