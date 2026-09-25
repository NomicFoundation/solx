//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use tempfile::TempDir;

#[test]
#[ignore = "the Slang frontend does not emit this output yet"]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT, "--debug-info"];

    let result = crate::cli::execute_solx(args)?;

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
        crate::common::TEST_SOLIDITY_STANDARD_JSON,
        "--debug-info",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Cannot output data outside of JSON in standard JSON mode.",
    ));

    Ok(())
}

#[test]
#[ignore = "the Slang frontend does not emit this output yet"]
fn output_dir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_debug_output")?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
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
