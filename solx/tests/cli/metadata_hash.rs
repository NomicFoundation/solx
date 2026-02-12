//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use solx_utils::MetadataHashType;

#[test]
fn none() -> anyhow::Result<()> {
    crate::common::setup()?;

    let hash_type = MetadataHashType::None.to_string();
    let args = &[
        "--metadata-hash",
        hash_type.as_str(),
        "--bin",
        crate::common::TEST_SOLIDITY_CONTRACT,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("a164"));

    Ok(())
}

#[test]
fn ipfs() -> anyhow::Result<()> {
    crate::common::setup()?;

    let hash_type = MetadataHashType::IPFS.to_string();
    let args = &[
        "--metadata-hash",
        hash_type.as_str(),
        "--bin",
        crate::common::TEST_SOLIDITY_CONTRACT,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("a264"));

    Ok(())
}

#[test]
fn standard_json_cli_excess_arg() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON,
        "--metadata-hash",
        "ipfs",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Metadata hash mode must be specified in standard JSON input settings.",
    ));

    Ok(())
}
