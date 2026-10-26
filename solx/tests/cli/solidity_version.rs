//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use test_case::test_case;

/// The `settings.solidityVersion` of contract `C` in source `A` of the standard JSON output.
fn standard_json_solidity_version(input: &str) -> anyhow::Result<serde_json::Value> {
    let result = crate::cli::execute_solx(&["--standard-json", input])?;
    let output = solx_utils::deserialize_from_slice::<solx_standard_json::Output>(
        result.success().get_output().stdout.as_slice(),
    )?;
    let metadata = output.contracts["A"]["C"]
        .metadata
        .as_deref()
        .expect("the metadata is selected");
    let metadata: serde_json::Value = serde_json::from_str(metadata)?;
    Ok(metadata["settings"]["solidityVersion"].clone())
}

#[test]
fn pinned_pragma() -> anyhow::Result<()> {
    crate::common::setup()?;

    let contract = crate::common::contract!("solidity/PinnedPragma.sol");
    let output = crate::cli::execute_solx_to_directory(&[
        "--solidity-version",
        "0.8.20",
        "--metadata",
        contract,
    ])?;
    let metadata = crate::cli::output_metadata(&output, contract, "PinnedPragma")?;

    assert_eq!(metadata["settings"]["solidityVersion"], "0.8.20");

    Ok(())
}

#[test]
fn inferred_from_pragma() -> anyhow::Result<()> {
    crate::common::setup()?;

    let contract = crate::common::contract!("solidity/PinnedPragma.sol");
    let output = crate::cli::execute_solx_to_directory(&["--metadata", contract])?;
    let metadata = crate::cli::output_metadata(&output, contract, "PinnedPragma")?;

    assert_eq!(metadata["settings"]["solidityVersion"], "0.8.20");

    Ok(())
}

#[test]
fn inferred_across_sources() -> anyhow::Result<()> {
    crate::common::setup()?;

    let caret = crate::common::contract!("solidity/PragmaCaret.sol");
    let range = crate::common::contract!("solidity/PragmaRange.sol");
    let output = crate::cli::execute_solx_to_directory(&["--metadata", caret, range])?;

    for (contract, name) in [(caret, "PragmaCaret"), (range, "PragmaRange")] {
        let metadata = crate::cli::output_metadata(&output, contract, name)?;
        assert_eq!(metadata["settings"]["solidityVersion"], "0.8.24");
    }

    Ok(())
}

#[test]
fn inferred_no_common_version() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--bin",
        crate::common::contract!("solidity/PinnedPragma.sol"),
        crate::common::contract!("solidity/PragmaCaret.sol"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "This pragma is incompatible with the compilation language version",
    ));

    Ok(())
}

#[test_case("0.8.20+commit.a1b79de6" ; "build")]
#[test_case("0.8.20-nightly.2023.5.1" ; "prerelease")]
#[test_case("0.8.20-nightly.2023.5.1+commit.a1b79de6" ; "prerelease_and_build")]
fn suffix(version: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let contract = crate::common::contract!("solidity/PinnedPragma.sol");
    let output = tempfile::TempDir::with_prefix("solx_output")?;
    let args = &[
        "--solidity-version",
        version,
        "--metadata",
        contract,
        "--output-dir",
        output.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stderr(predicate::str::contains(format!(
        "Solidity version {version} is compiled as 0.8.20"
    )));
    let metadata = crate::cli::output_metadata(&output, contract, "PinnedPragma")?;
    assert_eq!(metadata["settings"]["solidityVersion"], "0.8.20");

    Ok(())
}

#[test]
fn syntax_newer_than_version() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--solidity-version",
        "0.8.26",
        "--bin",
        crate::common::contract!("solidity/TransientStorage.sol"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "This syntax was introduced in version '0.8.27'.",
    ));

    Ok(())
}

#[test]
fn default_evm_version() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--solidity-version",
        "0.8.30",
        "--emit-mlir=sol",
        crate::common::TEST_SOLIDITY_CONTRACT,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("sol.evm_version = #Prague"));

    Ok(())
}

#[test]
fn default_evm_version_unsupported() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--solidity-version",
        "0.8.24",
        "--emit-mlir=sol",
        crate::common::TEST_SOLIDITY_CONTRACT,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("sol.evm_version = #Cancun"))
        .stderr(predicate::str::contains(
            "Solidity version 0.8.24 defaults to EVM version shanghai, which the compiler does not support yet. Compiling for cancun",
        ));

    Ok(())
}

#[test_case("0.7.6")]
#[test_case("0.9.0")]
fn unsupported(version: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--solidity-version",
        version,
        "--bin",
        crate::common::TEST_SOLIDITY_CONTRACT,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(format!(
        "Solidity version {version} is not supported."
    )));

    Ok(())
}

#[test]
fn yul() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--solidity-version",
        "0.8.20",
        "--yul",
        "--bin",
        crate::common::TEST_YUL_CONTRACT,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "Solidity version is only allowed in Solidity mode",
    ));

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    assert_eq!(
        standard_json_solidity_version(crate::common::standard_json!("solidity_version.json"))?,
        "0.8.20"
    );

    Ok(())
}

#[test]
fn standard_json_inferred() -> anyhow::Result<()> {
    crate::common::setup()?;

    assert_eq!(
        standard_json_solidity_version(crate::common::standard_json!(
            "solidity_version_inferred.json"
        ))?,
        "0.8.20"
    );

    Ok(())
}

#[test]
fn standard_json_unsupported() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_version_unsupported.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Solidity version 0.7.6 is not supported.",
    ));

    Ok(())
}

#[test]
fn standard_json_with_flag() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_version.json"),
        "--solidity-version",
        "0.8.20",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Solidity version must be passed via standard JSON input.",
    ));

    Ok(())
}
