//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT, "--metadata"];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("Metadata").count(1));

    Ok(())
}

#[test]
fn invalid_input() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_YUL_CONTRACT, "--metadata"];

    let result = crate::cli::execute_solx(args)?;

    result
        .failure()
        .stderr(predicate::str::contains("Unexpected StringLiteral"));

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON,
        "--metadata",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Cannot output data outside of JSON in standard JSON mode.",
    ));

    Ok(())
}

#[test]
fn sources_import_closure() -> anyhow::Result<()> {
    crate::common::setup()?;

    let main = crate::common::contract!("solidity/caller/Main.sol");
    let callable = crate::common::contract!("solidity/caller/Callable.sol");
    let unrelated = crate::common::TEST_SOLIDITY_CONTRACT;
    let output = crate::cli::execute_solx_to_directory(&[main, callable, unrelated, "--metadata"])?;

    assert_eq!(
        source_names(&crate::cli::output_metadata(&output, main, "Main")?),
        [callable, main]
    );
    assert_eq!(
        source_names(&crate::cli::output_metadata(&output, callable, "Callable")?),
        [callable]
    );

    Ok(())
}

#[test]
fn defaults() -> anyhow::Result<()> {
    crate::common::setup()?;

    let contract = crate::common::TEST_SOLIDITY_CONTRACT;
    let output = crate::cli::execute_solx_to_directory(&[contract, "--metadata"])?;
    let metadata = crate::cli::output_metadata(&output, contract, "SlangTest")?;
    let solc_abi: serde_json::Value = serde_json::from_str(
        std::fs::read_to_string(crate::common::abi!("SlangTest.json"))?.as_str(),
    )?;

    assert_eq!(
        metadata["settings"],
        serde_json::json!({
            "compilationTarget": { contract: "SlangTest" },
            "evmVersion": "osaka",
            "libraries": {},
            "metadata": { "bytecodeHash": "ipfs" },
            "optimizer": { "mode": "3", "sizeFallback": false },
            "remappings": [],
            "solidityVersion": metadata["settings"]["solidityVersion"],
        })
    );
    assert_eq!(metadata["output"]["abi"], solc_abi["SlangTest"]);

    Ok(())
}

#[test]
fn settings() -> anyhow::Result<()> {
    crate::common::setup()?;

    let contract = crate::common::TEST_SOLIDITY_CONTRACT;
    let library = "tests/data/contracts/solidity/MiniMath.sol:MiniMath";
    let address = "0xF9702469Dfb84A9aC171E284F71615bd3D3f1EdC";
    let output = crate::cli::execute_solx_to_directory(&[
        contract,
        "b/=c/",
        "a/=b/",
        "b/=c/",
        "--metadata",
        "--metadata-hash",
        "none",
        "--no-cbor-metadata",
        "-Oz",
        "--optimization-size-fallback",
        "--llvm-options=-evm-metadata-size 10",
        "--evm-version",
        "cancun",
        "--solidity-version",
        "0.8.30",
        "--libraries",
        format!("{library}={address}").as_str(),
    ])?;
    let metadata = crate::cli::output_metadata(&output, contract, "SlangTest")?;
    let content = std::fs::read(contract)?;

    assert_eq!(
        metadata["compiler"],
        serde_json::json!({ "name": "slang", "version": solx_core::Compiler::version() })
    );
    assert_eq!(metadata["language"], "Solidity");
    assert_eq!(metadata["version"], 1);
    assert_eq!(
        metadata["settings"],
        serde_json::json!({
            "compilationTarget": { contract: "SlangTest" },
            "evmVersion": "cancun",
            "libraries": { library: address.to_lowercase() },
            "llvmOptions": ["-evm-metadata-size", "10"],
            "metadata": { "appendCBOR": false, "bytecodeHash": "none" },
            "optimizer": { "mode": "z", "sizeFallback": true },
            "remappings": [":a/=b/", ":b/=c/"],
            "solidityVersion": "0.8.30",
        })
    );
    assert_eq!(
        metadata["sources"][contract]["keccak256"],
        solx_utils::Keccak256Hash::from_slice(content.as_slice()).as_str()
    );
    assert_eq!(
        metadata["sources"][contract]["urls"],
        serde_json::json!([format!(
            "dweb:/ipfs/{}",
            solx_utils::IPFSHash::from_slice(content.as_slice()).as_base58()
        )])
    );

    Ok(())
}

#[test]
fn revert_strings() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("revert_strings_strip.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    let output = solx_utils::deserialize_from_slice::<solx_standard_json::Output>(
        result.success().get_output().stdout.as_slice(),
    )?;
    let metadata: serde_json::Value = serde_json::from_str(
        output.contracts["A"]["C"]
            .metadata
            .as_deref()
            .expect("the metadata is selected"),
    )?;

    assert_eq!(
        metadata["settings"]["debug"],
        serde_json::json!({ "revertStrings": "strip" })
    );

    Ok(())
}

/// The names in `metadata`'s `sources`, in their sorted order.
fn source_names(metadata: &serde_json::Value) -> Vec<String> {
    metadata["sources"]
        .as_object()
        .map(|sources| sources.keys().cloned().collect())
        .unwrap_or_default()
}
