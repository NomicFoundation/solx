//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT, "--debug-info"];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("Debug info:\n7f454c46").count(1))
        .stdout(predicate::str::contains("runtime part").not());

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

    let output_file = format!(
        "{}_SlangTest.dbg.{}",
        crate::common::TEST_SOLIDITY_CONTRACT.replace(['\\', '/', '.'], "_"),
        solx_utils::EXTENSION_EVM_BINARY,
    );
    let debug_info = std::fs::read(output_directory.path().join(output_file))?;
    assert!(debug_info.starts_with(b"\x7fELF"));

    Ok(())
}
