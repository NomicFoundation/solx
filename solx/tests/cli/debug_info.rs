//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use tempfile::TempDir;
use test_case::test_case;

#[test_case(true ; "yul")]
#[test_case(false ; "evmla")]
fn default(via_ir: bool) -> anyhow::Result<()> {
    crate::common::setup()?;

    let mut args = vec![crate::common::TEST_SOLIDITY_CONTRACT_PATH, "--debug-info"];
    if via_ir {
        args.push("--via-ir");
    }

    let result = crate::cli::execute_solx(&args)?;

    result
        .success()
        .stdout(predicate::str::contains("Debug info").count(1));

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON_PATH,
        "--debug-info",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Cannot output data outside of JSON in standard JSON mode.",
    ));

    Ok(())
}

#[test]
fn output_dir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_debug_output")?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--debug-info",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    Ok(())
}
