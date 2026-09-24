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
    #[cfg(feature = "solc")]
    result.success().stdout(predicate::str::contains("a264"));
    #[cfg(not(feature = "solc"))]
    result.success().stdout(predicate::str::contains("a164"));

    Ok(())
}

#[test]
fn none_prints_compiler_section() -> anyhow::Result<()> {
    crate::common::setup()?;

    let hash_type = MetadataHashType::None.to_string();
    let args = &[
        crate::common::TEST_YUL_CONTRACT,
        "--yul",
        "--metadata",
        "--metadata-hash",
        hash_type.as_str(),
    ];

    let result = crate::cli::execute_solx(args)?;
    #[cfg(feature = "solc")]
    result
        .success()
        .stdout(predicate::str::contains("\"solx\":{"));
    #[cfg(not(feature = "solc"))]
    result
        .success()
        .stdout(predicate::str::contains("\"slang\":{"));

    Ok(())
}

#[test]
fn ipfs_hashes_printed_metadata() -> anyhow::Result<()> {
    crate::common::setup()?;

    let hash_type = MetadataHashType::IPFS.to_string();
    let args = &[
        crate::common::TEST_YUL_CONTRACT,
        "--yul",
        "--metadata",
        "--metadata-hash",
        hash_type.as_str(),
        "--bin-runtime",
    ];

    let result = crate::cli::execute_solx(args)?;
    let stdout = String::from_utf8(result.success().get_output().stdout.clone())?;
    let line_after = |header: &str| {
        stdout
            .lines()
            .skip_while(|line| *line != header)
            .nth(1)
            .unwrap_or_else(|| panic!("`{header}` is followed by its value"))
    };
    let runtime = line_after("Binary of the runtime part:");
    let metadata = line_after("Metadata:");

    // A two-entry CBOR map whose first entry is `ipfs` with a 34-byte value.
    let (_, hash) = runtime
        .rsplit_once("a264697066735822")
        .expect("the runtime code ends with an IPFS metadata hash");
    assert_eq!(
        &hash[..68],
        solx_utils::IPFSHash::from_slice(metadata.as_bytes()).to_string()
    );

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
