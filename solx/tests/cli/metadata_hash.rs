//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use solx_utils::MetadataHashType;

/// A two-entry CBOR map whose first entry is `ipfs` with a 34-byte value.
const CBOR_IPFS_HASH_PREFIX: &str = "a264697066735822";

/// The key of the compiler's own section in the metadata.
#[cfg(feature = "solc")]
const SECTION_KEY: &str = "\"solx\":{";
/// The key of the compiler's own section in the metadata.
#[cfg(not(feature = "solc"))]
const SECTION_KEY: &str = "\"slang\":{";

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
        "--bin-runtime",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains(SECTION_KEY))
        .stdout(predicate::str::contains(CBOR_IPFS_HASH_PREFIX).not());

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

    let (_, hash) = runtime
        .rsplit_once(CBOR_IPFS_HASH_PREFIX)
        .expect("the runtime code ends with an IPFS metadata hash");
    assert_eq!(
        &hash[..68],
        solx_utils::IPFSHash::from_slice(metadata.as_bytes()).to_string()
    );

    Ok(())
}

#[test]
fn ipfs_hashes_standard_json_metadata() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("llvm_ir_metadata.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    let output = solx_utils::deserialize_from_slice::<solx_standard_json::Output>(
        result.success().get_output().stdout.as_slice(),
    )?;
    let contract = &output.contracts["Test"]["Test"];
    let metadata = contract
        .metadata
        .as_deref()
        .expect("the metadata is selected");
    let runtime = contract
        .evm
        .as_ref()
        .and_then(|evm| evm.deployed_bytecode.as_ref())
        .and_then(|bytecode| bytecode.object.as_deref())
        .expect("the runtime bytecode is selected");

    assert!(
        metadata.contains(SECTION_KEY),
        "the metadata carries the compiler's own section"
    );
    let (_, hash) = runtime
        .rsplit_once(CBOR_IPFS_HASH_PREFIX)
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
