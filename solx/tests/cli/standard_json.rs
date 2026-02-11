//!
//! CLI tests for the eponymous option.
//!

use std::io::Read;

use predicates::prelude::*;
use test_case::test_case;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity.json"),
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
        crate::cli::execute_solx_with_stdin(args, crate::common::standard_json!("solidity.json"))?;
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
        crate::cli::execute_solx_with_stdin(args, crate::common::standard_json!("solidity.json"))?;
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
        crate::common::standard_json!("solidity_deploy_time_linking.json"),
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
        crate::common::standard_json!("solidity_recursion.json"),
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
        crate::common::standard_json!("solidity_fuzzed_simple_use_expression.json"),
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

    let args = &["--standard-json", crate::common::contract!("yul/Test.yul")];

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
        crate::common::standard_json!("solidity_invalid.json"),
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
        crate::common::standard_json!("non_existent.json"),
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
        crate::common::standard_json!("invalid_utf8.json"),
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
        crate::common::standard_json!("solidity_empty_sources.json"),
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
        crate::common::standard_json!("solidity_missing_sources.json"),
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
        crate::common::standard_json!("metadata_hash_ipfs_and_metadata.json"),
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
        crate::common::standard_json!("metadata_hash_ipfs_no_metadata.json"),
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
        crate::common::standard_json!("metadata_hash_none_and_metadata.json"),
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
        crate::common::standard_json!("metadata_hash_none_no_metadata.json"),
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
        crate::common::standard_json!("select_evm.json"),
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
        crate::common::standard_json!("select_evm_bytecode.json"),
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
        crate::common::standard_json!("select_evm_deployed_bytecode.json"),
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
        crate::common::standard_json!("select_evm_bytecode_opcodes.json"),
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
        crate::common::standard_json!("select_evm_deployed_bytecode_link_references.json"),
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
        crate::common::standard_json!("select_single.json"),
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

#[test_case(crate::common::standard_json!("select_all.json"))]
#[test_case(crate::common::standard_json!("select_all_wildcard.json"))]
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

#[test]
fn debug_env_writes_input() -> anyhow::Result<()> {
    crate::common::setup()?;

    let temp_dir = tempfile::tempdir()?;
    let debug_path = temp_dir.path().join("debug_input.json");

    let input_path = crate::common::standard_json!("solidity.json");
    let args = &["--standard-json", input_path];

    let result = crate::cli::execute_solx_with_env_vars(
        args,
        vec![(
            solx_standard_json::STANDARD_JSON_DEBUG_ENV,
            debug_path.to_string_lossy().to_string(),
        )],
    )?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"));

    let mut debug_content = String::new();
    std::fs::File::open(&debug_path)?.read_to_string(&mut debug_content)?;
    let original_content = std::fs::read_to_string(input_path)?;
    assert_eq!(debug_content, original_content);

    Ok(())
}

#[test]
fn select_llvm_ir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("select_llvm_ir.json"),
    )?;

    result.success().stdout(
        predicate::str::contains("llvmIr")
            .and(predicate::str::contains("llvmIrUnoptimized"))
            .and(predicate::str::contains("object")),
    );

    Ok(())
}

#[test]
fn select_evmla_ethir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("select_evmla_ethir.json"),
    )?;

    result.success().stdout(
        predicate::str::contains("evmla")
            .and(predicate::str::contains("ethir"))
            .and(predicate::str::contains("object")),
    );

    Ok(())
}
