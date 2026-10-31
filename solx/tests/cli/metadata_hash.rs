//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use solx_utils::MetadataHashType;

/// The suffix of the runtime code's file in `--output-dir`.
const RUNTIME_SUFFIX: &str = ".bin-runtime";

/// The suffix of the metadata's file in `--output-dir`.
const METADATA_SUFFIX: &str = "_meta.json";

/// The compiler's entry in the metadata.
const COMPILER_KEY: &str = "\"compiler\":{\"name\":\"slang\"";

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
    result.success().stdout(predicate::str::contains(
        crate::common::CBOR_WITHOUT_HASH_PREFIX,
    ));

    Ok(())
}

#[test]
fn ipfs_distinguishes_identical_code() -> anyhow::Result<()> {
    crate::common::setup()?;

    let contract = crate::common::contract!("solidity/InheritedCode.sol");
    let output = crate::cli::execute_solx_to_directory(&[contract, "--bin-runtime"])?;
    let runtime = |name: &str| crate::cli::output_file(&output, contract, name, RUNTIME_SUFFIX);

    assert_ne!(runtime("Base")?, runtime("Derived")?);

    Ok(())
}

#[test]
fn none_prints_compiler() -> anyhow::Result<()> {
    crate::common::setup()?;

    let hash_type = MetadataHashType::None.to_string();
    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--metadata",
        "--metadata-hash",
        hash_type.as_str(),
        "--bin-runtime",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains(COMPILER_KEY))
        .stdout(predicate::str::contains(crate::common::CBOR_IPFS_HASH_PREFIX).not());

    Ok(())
}

#[test]
fn ipfs_hashes_cli_metadata() -> anyhow::Result<()> {
    crate::common::setup()?;

    let contract = crate::common::TEST_SOLIDITY_CONTRACT;
    let hash_type = MetadataHashType::IPFS.to_string();
    let output = crate::cli::execute_solx_to_directory(&[
        contract,
        "--metadata",
        "--metadata-hash",
        hash_type.as_str(),
        "--bin-runtime",
    ])?;
    let runtime = crate::cli::output_file(&output, contract, "SlangTest", RUNTIME_SUFFIX)?;
    let metadata = crate::cli::output_file(&output, contract, "SlangTest", METADATA_SUFFIX)?;

    let (_, hash) = runtime
        .rsplit_once(crate::common::CBOR_IPFS_HASH_PREFIX)
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
        metadata.contains(COMPILER_KEY),
        "the metadata names the compiler"
    );
    let (_, hash) = runtime
        .rsplit_once(crate::common::CBOR_IPFS_HASH_PREFIX)
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
