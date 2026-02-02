//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use test_case::test_case;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"))
        .stdout(predicate::str::contains("object"))
        .stdout(predicate::str::contains("debugInfo"));

    Ok(())
}

#[test]
fn stdin() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &["--standard-json"];

    let result =
        crate::cli::execute_solx_with_stdin(args, crate::common::TEST_SOLIDITY_STANDARD_JSON_PATH)?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"))
        .stdout(predicate::str::contains("object"))
        .stdout(predicate::str::contains("debugInfo"));

    Ok(())
}

#[test]
fn stdin_hyphen() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        solx_standard_json::InputSource::STDIN_INPUT_IDENTIFIER,
    ];

    let result =
        crate::cli::execute_solx_with_stdin(args, crate::common::TEST_SOLIDITY_STANDARD_JSON_PATH)?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"))
        .stdout(predicate::str::contains("object"))
        .stdout(predicate::str::contains("debugInfo"));

    Ok(())
}

#[test]
fn deploy_time_linking() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON_DEPLOY_TIME_LINKING_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("__$32d65841735fc578113c8cbc3571729a2b$__").count(2));

    Ok(())
}

#[test]
fn recursion() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON_RECURSION_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"))
        .stdout(predicate::str::contains("object"))
        .stdout(predicate::str::contains("debugInfo"));

    Ok(())
}

#[test]
fn fuzzed_simple_use_expression() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON_FUZZED_SIMPLE_USE_EXPRESSION_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"))
        .stdout(predicate::str::contains("object"))
        .stdout(predicate::str::contains("debugInfo"));

    Ok(())
}

#[test]
fn invalid_input() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &["--standard-json", crate::common::TEST_YUL_CONTRACT_PATH];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("parsing: expected value"));

    Ok(())
}

#[test]
fn invalid_input_solc_error() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON_INVALID_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "ParserError: Expected identifier but got",
    ));

    Ok(())
}

#[test]
fn invalid_path() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON_NON_EXISTENT_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains(
            "Standard JSON file \\\"tests/data/standard_json_input/non_existent.json\\\" reading",
        ))
        .code(solx_utils::EXIT_CODE_SUCCESS);

    Ok(())
}

#[test]
fn invalid_utf8() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON_INVALID_UTF8_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Standard JSON parsing: expected value",
    ));

    Ok(())
}

#[test]
fn stdin_missing() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &["--standard-json"];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Standard JSON parsing: EOF while parsing",
    ));

    Ok(())
}

#[test]
fn stdin_hyphen_missing() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        solx_standard_json::InputSource::STDIN_INPUT_IDENTIFIER,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Standard JSON parsing: EOF while parsing",
    ));

    Ok(())
}

#[test]
fn empty_sources() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON_EMPTY_SOURCES_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("No input sources specified."));

    Ok(())
}

#[test]
fn missing_sources() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON_MISSING_SOURCES_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Standard JSON parsing: missing field `sources`",
    ));

    Ok(())
}

#[test]
fn metadata_hash_ipfs_and_metadata() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_JSON_METADATA_HASH_IPFS_AND_METADATA,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("a264"))
        .stdout(predicate::str::contains("\"metadata\""));

    Ok(())
}

#[test]
fn metadata_hash_ipfs_no_metadata() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_JSON_METADATA_HASH_IPFS_NO_METADATA,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("a264"))
        .stdout(predicate::str::contains("\"metadata\"").not());

    Ok(())
}

#[test]
fn metadata_hash_none_and_metadata() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_JSON_METADATA_HASH_NONE_AND_METADATA,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("a164"))
        .stdout(predicate::str::contains("\"metadata\""));

    Ok(())
}

#[test]
fn metadata_hash_none_no_metadata() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_JSON_METADATA_HASH_NONE_NO_METADATA,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("a164"))
        .stdout(predicate::str::contains("\"metadata\"").not());

    Ok(())
}

#[test]
fn select_evm() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON_SELECT_EVM_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"))
        .stdout(predicate::str::contains("deployedBytecode"))
        .stdout(predicate::str::contains("debugInfo"))
        .stdout(predicate::str::contains("llvmAssembly"))
        .stdout(predicate::str::contains("opcodes"))
        .stdout(predicate::str::contains("linkReferences"));

    Ok(())
}

#[test]
fn select_evm_bytecode() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON_SELECT_EVM_BYTECODE_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"))
        .stdout(predicate::str::contains("deployedBytecode").not())
        .stdout(predicate::str::contains("metadata").not());

    Ok(())
}

#[test]
fn select_evm_deployed_bytecode() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON_SELECT_EVM_DEPLOYED_BYTECODE_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("deployedBytecode"))
        .stdout(predicate::str::contains("bytecode").not())
        .stdout(predicate::str::contains("metadata").not());

    Ok(())
}

#[test]
fn select_evm_bytecode_opcodes() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON_SELECT_EVM_BYTECODE_OPCODES_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("opcodes"))
        .stdout(predicate::str::contains("deployedBytecode").not());

    Ok(())
}

#[test]
fn select_evm_deployed_bytecode_link_references() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON_SELECT_EVM_DEPLOYED_BYTECODE_LINK_REFERENCES_PATH
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("linkReferences"))
        .stdout(predicate::str::contains("bytecode").not());

    Ok(())
}

#[test]
fn select_single() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON_SELECT_SINGLE_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"bytecode\"").count(1))
        .stdout(predicate::str::contains("\"deployedBytecode\"").count(1));

    Ok(())
}

#[test]
fn select_none() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON_SELECT_NONE_PATH,
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"evm\"").not())
        .stdout(predicate::str::contains("\"bytecode\"").not())
        .stdout(predicate::str::contains("\"deployedBytecode\"").not());

    Ok(())
}

#[test_case(crate::common::TEST_SOLIDITY_STANDARD_JSON_SELECT_ALL_PATH)]
#[test_case(crate::common::TEST_SOLIDITY_STANDARD_JSON_SELECT_ALL_WILDCARD_PATH)]
fn select_all(path: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &["--standard-json", path];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"ast\""))
        .stdout(predicate::str::contains("\"abi\""))
        .stdout(predicate::str::contains("\"metadata\""))
        .stdout(predicate::str::contains("\"devdoc\""))
        .stdout(predicate::str::contains("\"userdoc\""))
        .stdout(predicate::str::contains("\"storageLayout\""))
        .stdout(predicate::str::contains("\"transientStorageLayout\""))
        .stdout(predicate::str::contains("\"methodIdentifiers\""))
        .stdout(predicate::str::contains("\"legacyAssembly\""))
        .stdout(predicate::str::contains("\"gasEstimates\""))
        .stdout(predicate::str::contains("\"ir\""))
        .stdout(predicate::str::contains("\"benchmarks\"").count(3))
        .stdout(predicate::str::contains("\"evm\""))
        .stdout(predicate::str::contains("\"bytecode\""))
        .stdout(predicate::str::contains("\"deployedBytecode\""))
        .stdout(predicate::str::contains("\"object\"").count(2))
        .stdout(predicate::str::contains("\"llvmAssembly\"").count(2))
        .stdout(predicate::str::contains("\"opcodes\"").count(2))
        .stdout(predicate::str::contains("\"linkReferences\"").count(2))
        .stdout(predicate::str::contains("\"debugInfo\"").count(2))
        .stdout(predicate::str::contains("\"sourceMap\"").count(2))
        .stdout(predicate::str::contains("\"functionDebugData\"").count(2))
        .stdout(predicate::str::contains("\"generatedSources\"").count(2))
        .stdout(predicate::str::contains("\"immutableReferences\""));

    Ok(())
}
