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
    let output =
        crate::cli::execute_solx_to_directory(&[contract, "--metadata", "--metadata-literal"])?;
    let metadata = crate::cli::output_metadata(&output, contract, "SlangTest")?;
    let source = &metadata["sources"][contract];
    let content = std::fs::read_to_string(contract)?;

    assert_eq!(
        metadata["settings"]["metadata"],
        serde_json::json!({ "bytecodeHash": "ipfs", "useLiteralContent": true })
    );
    assert_eq!(source["content"].as_str(), Some(content.as_str()));
    assert_eq!(
        source["keccak256"],
        solx_utils::Keccak256Hash::from_slice(content.as_bytes()).as_str()
    );
    assert!(source.get("urls").is_none());

    Ok(())
}
