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
    let metadata = crate::cli::metadata(&[main, callable, unrelated, "--metadata"])?;

    assert_eq!(source_names(&metadata["Main"]), [callable, main]);
    assert_eq!(source_names(&metadata["Callable"]), [callable]);

    Ok(())
}

#[test]
fn settings() -> anyhow::Result<()> {
    crate::common::setup()?;

    let contract = crate::common::TEST_SOLIDITY_CONTRACT;
    let library = "tests/data/contracts/solidity/MiniMath.sol:MiniMath";
    let address = "0xF9702469Dfb84A9aC171E284F71615bd3D3f1EdC";
    let metadata = crate::cli::metadata(&[
        contract,
        "b/=c/",
        "a/=b/",
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
    let metadata = &metadata["SlangTest"];
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
            "libraries": { library: address },
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

/// The names in `metadata`'s `sources`, in their sorted order.
fn source_names(metadata: &serde_json::Value) -> Vec<&str> {
    metadata["sources"]
        .as_object()
        .map(|sources| sources.keys().map(String::as_str).collect())
        .unwrap_or_default()
}
