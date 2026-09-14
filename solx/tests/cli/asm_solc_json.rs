//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT, "--asm-solc-json"];

    let result = crate::cli::execute_solx(args)?;

    #[cfg(feature = "solc")]
    result
        .success()
        .stdout(predicate::str::contains("EVM assembly").count(1));
    #[cfg(not(feature = "solc"))]
    result
        .failure()
        .stderr(predicate::str::contains("--asm-solc-json is not honored"))
        .stderr(predicate::str::contains("Nothing would be produced"));

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON,
        "--asm-solc-json",
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

    let output_directory = TempDir::with_prefix("solx_evmasm_output")?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--asm-solc-json",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    #[cfg(feature = "solc")]
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));
    #[cfg(not(feature = "solc"))]
    result
        .failure()
        .stderr(predicate::str::contains("Nothing would be produced"));

    Ok(())
}
