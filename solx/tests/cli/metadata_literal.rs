//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--metadata-literal",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Binary:\n"));

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON,
        "--metadata-literal",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Metadata literal content flag must be specified in standard JSON input settings.",
    ));

    Ok(())
}

#[test]
fn content() -> anyhow::Result<()> {
    crate::common::setup()?;

    let contract = crate::common::TEST_SOLIDITY_CONTRACT;
    let metadata = crate::cli::metadata(&[contract, "--metadata", "--metadata-literal"])?;
    let source = &metadata["SlangTest"]["sources"][contract];

    assert_eq!(
        source["content"].as_str(),
        Some(std::fs::read_to_string(contract)?.as_str())
    );
    assert!(source.get("urls").is_none());

    Ok(())
}
