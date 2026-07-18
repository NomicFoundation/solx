//!
//! CLI tests for standard JSON output structure and format.
//!
//! These tests exercise code paths in `solx-standard-json/src/output/mod.rs`
//! including output pruning, method identifiers, multi-contract output, etc.
//!

use predicates::prelude::*;
use test_case::test_case;

#[cfg(feature = "solc")]
#[test]
fn method_identifiers() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_method_identifiers.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"methodIdentifiers\""))
        .stdout(predicate::str::contains("foo()"))
        .stdout(predicate::str::contains("bar(uint256)"))
        .stdout(predicate::str::contains("\"object\""));

    Ok(())
}

#[cfg(feature = "solc")]
#[test]
fn multi_contract() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_multi_contract.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"First\""))
        .stdout(predicate::str::contains("\"Second\""))
        .stdout(predicate::str::contains("\"methodIdentifiers\""))
        .stdout(predicate::str::contains("\"opcodes\""));

    Ok(())
}

#[test]
fn multi_contract_bytecodes() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_multi_contract.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"bytecode\""))
        .stdout(predicate::str::contains("\"deployedBytecode\""))
        .stdout(predicate::str::contains("\"sourceMap\""));

    Ok(())
}

#[test]
fn gas_estimates() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_gas_estimates.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"gasEstimates\""))
        .stdout(predicate::str::contains("\"object\""));

    Ok(())
}

#[test_case(crate::common::TEST_SOLIDITY_STANDARD_JSON)]
#[test_case(crate::common::standard_json!("solidity_method_identifiers.json"))]
#[test_case(crate::common::standard_json!("solidity_multi_contract.json"))]
fn output_has_no_errors(path: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &["--standard-json", path];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"severity\":\"error\"").not());

    Ok(())
}

#[cfg(feature = "solc")]
#[test]
fn yul_standard_json_output_has_contracts() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &["--standard-json", crate::common::standard_json!("yul.json")];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"contracts\""))
        .stdout(predicate::str::contains("\"object\""))
        .stdout(predicate::str::contains("\"llvmAssembly\""));

    Ok(())
}

#[cfg(feature = "solc")]
#[test_case(crate::common::standard_json!("metadata_hash_ipfs_and_metadata.json"), true, true)]
#[test_case(crate::common::standard_json!("metadata_hash_ipfs_no_metadata.json"), true, false)]
#[test_case(crate::common::standard_json!("metadata_hash_none_and_metadata.json"), false, true)]
#[test_case(crate::common::standard_json!("metadata_hash_none_no_metadata.json"), false, false)]
fn metadata_hash_variants(
    path: &str,
    expect_ipfs_marker: bool,
    expect_metadata: bool,
) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &["--standard-json", path];

    let result = crate::cli::execute_solx(args)?;
    let mut assertion = result.success();
    if expect_ipfs_marker {
        assertion = assertion.stdout(predicate::str::contains("a264"));
    } else {
        assertion = assertion.stdout(predicate::str::contains("a164"));
    }
    if expect_metadata {
        assertion.stdout(predicate::str::contains("\"metadata\""));
    } else {
        assertion.stdout(predicate::str::contains("\"metadata\"").not());
    }

    Ok(())
}

#[cfg(feature = "solc")]
#[test]
fn error_output_has_formatted_message() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_invalid.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"formattedMessage\""))
        .stdout(predicate::str::contains("\"severity\""))
        .stdout(predicate::str::contains("\"type\""))
        .stdout(predicate::str::contains("ParserError"));

    Ok(())
}

#[test]
fn error_output_has_source_location() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_invalid.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"sourceLocation\""))
        .stdout(predicate::str::contains("\"file\""))
        .stdout(predicate::str::contains("\"start\""))
        .stdout(predicate::str::contains("\"end\""));

    Ok(())
}

#[test]
fn error_output_component_is_general() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_invalid.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"component\":\"general\""));

    Ok(())
}

#[cfg_attr(
    not(feature = "solc"),
    test_case(crate::common::standard_json!("solidity_missing_sources.json"), "missing field `sources`")
)]
#[cfg_attr(
    feature = "solc",
    test_case(crate::common::standard_json!("solidity_empty_sources.json"), "No input sources specified")
)]
#[cfg_attr(
    feature = "solc",
    test_case(crate::common::standard_json!("solidity_missing_sources.json"), "missing field `sources`")
)]
fn error_messages(path: &str, expected_message: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &["--standard-json", path];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains(expected_message));

    Ok(())
}

#[test]
fn warning_output_has_correct_severity() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON,
        "--optimization-size-fallback",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Size optimization fallback must be specified in standard JSON input settings.",
    ));

    Ok(())
}

#[test]
fn select_none_produces_empty_contracts() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_none.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"evm\"").not())
        .stdout(predicate::str::contains("\"bytecode\"").not())
        .stdout(predicate::str::contains("\"deployedBytecode\"").not());

    Ok(())
}

#[test_case(crate::common::standard_json!("select_evm_bytecode.json"), "bytecode")]
#[test_case(crate::common::standard_json!("select_evm_deployed_bytecode.json"), "deployedBytecode")]
fn select_specific_bytecode(path: &str, expected_key: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &["--standard-json", path];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains(expected_key));

    Ok(())
}

#[cfg(feature = "solc")]
#[test]
fn via_ir_output_structure() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_via_ir.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"abi\""))
        .stdout(predicate::str::contains("\"object\""))
        .stdout(predicate::str::contains("\"deployedBytecode\""))
        .stdout(predicate::str::contains("\"metadata\""));

    Ok(())
}

#[test]
fn both_content_and_urls_error() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_both_content_and_urls.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Both `content` and `urls` cannot be set",
    ));

    Ok(())
}

#[test]
fn remappings_in_standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_with_remappings.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"object\""))
        .stdout(predicate::str::contains("\"severity\":\"error\"").not());

    Ok(())
}

#[test]
fn evm_version_in_standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_with_evm_version.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"object\""))
        .stdout(predicate::str::contains("\"severity\":\"error\"").not());

    Ok(())
}

#[cfg(feature = "solc")]
#[test]
fn storage_layout_output() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_storage_layout.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"storageLayout\""))
        .stdout(predicate::str::contains("\"object\""));

    Ok(())
}

#[cfg(feature = "solc")]
#[test]
fn abi_only_output() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_abi_only.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"abi\""))
        .stdout(predicate::str::contains("\"name\":\"foo\""))
        .stdout(predicate::str::contains("\"name\":\"Transfer\""));

    Ok(())
}

#[cfg(feature = "solc")]
#[test]
fn devdoc_userdoc_output() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_devdoc_userdoc.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"devdoc\""))
        .stdout(predicate::str::contains("\"userdoc\""));

    Ok(())
}

#[test_case(crate::common::standard_json!("solidity_via_ir.json"))]
#[test_case(crate::common::standard_json!("solidity_with_remappings.json"))]
#[test_case(crate::common::standard_json!("solidity_with_evm_version.json"))]
#[test_case(crate::common::standard_json!("solidity_storage_layout.json"))]
#[test_case(crate::common::standard_json!("solidity_abi_only.json"))]
#[test_case(crate::common::standard_json!("solidity_devdoc_userdoc.json"))]
fn additional_outputs_no_errors(path: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &["--standard-json", path];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"severity\":\"error\"").not());

    Ok(())
}

#[test_case(crate::common::standard_json!("solidity_via_ir.json"))]
#[test_case(crate::common::standard_json!("solidity_with_remappings.json"))]
#[test_case(crate::common::standard_json!("solidity_with_evm_version.json"))]
fn additional_outputs_via_stdin(path: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &["--standard-json"];

    let result = crate::cli::execute_solx_with_stdin(args, path)?;
    result
        .success()
        .stdout(predicate::str::contains("\"object\""))
        .stdout(predicate::str::contains("\"severity\":\"error\"").not());

    Ok(())
}

#[test]
fn select_ast_only() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_ast_only.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"ast\""))
        .stdout(predicate::str::contains("\"evm\"").not())
        .stdout(predicate::str::contains("\"bytecode\"").not())
        .stdout(predicate::str::contains("\"abi\"").not());

    Ok(())
}
