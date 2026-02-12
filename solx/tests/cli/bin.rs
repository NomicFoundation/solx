//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use test_case::test_case;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT, "--bin"];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("Binary").count(1));

    Ok(())
}

#[test]
fn stdin() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--bin",
        solx_standard_json::InputSource::STDIN_INPUT_IDENTIFIER,
    ];

    let result = crate::cli::execute_solx_with_stdin(args, crate::common::TEST_SOLIDITY_CONTRACT)?;

    result
        .success()
        .stdout(predicate::str::contains("Binary").count(1));

    Ok(())
}

#[test_case(crate::common::contract!("solidity/SimpleContract.sol"), vec!["__$733ff2b5a7b9002c636c19ae8206a21f88$__"])]
#[test_case(crate::common::contract!("solidity/LinkedMixedDeps.sol"), vec!["__$65ec92bf84627f42eab2cb5e40b5cc19ff$__"])]
#[test_case(crate::common::contract!("solidity/LinkedMixedDepsMultiLevel.sol"), vec!["__$c1091a910937160002c95b60eab1fc9a86$__", "__$71eefe2b783075e8d047b21bbc2b61aa32$__"])]
fn deploy_time_linking(path: &str, placeholders: Vec<&str>) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[path, "--bin"];

    let mut result = crate::cli::execute_solx(args)?;

    result = result.success().stdout(predicate::str::contains("Binary"));
    for placeholder in placeholders.into_iter() {
        result = result.stdout(predicate::str::contains(placeholder));
    }

    Ok(())
}

#[test]
fn stack_too_deep_solc() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/StackTooDeepSolc.sol"),
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("Binary").count(1));

    Ok(())
}

#[test]
fn stack_too_deep_llvm() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/StackTooDeepLLVM.sol"),
        "--bin",
        "-O1",
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stderr(predicate::str::contains("Warning: Performance of this contract can be compromised due to the presence of this memory-unsafe assembly block."));

    Ok(())
}

#[test]
fn stack_too_deep_llvm_suppressed() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/StackTooDeepLLVM.sol"),
        "--bin",
        "-O1",
    ];
    let env_vars = vec![("EVM_DISABLE_MEMORY_SAFE_ASM_CHECK", "1".to_owned())];

    let result = crate::cli::execute_solx_with_env_vars(args, env_vars)?;

    result
        .success()
        .stdout(predicate::str::contains("Binary").count(2));

    Ok(())
}

#[test]
fn fuzzed_linker_error() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/FuzzedLinkerError.sol"),
        "--bin-runtime",
        "-O1",
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("Binary of the runtime part").count(3));

    Ok(())
}

#[test]
fn invalid_input() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_YUL_CONTRACT, "--bin"];

    let result = crate::cli::execute_solx(args)?;

    result.failure().stderr(predicate::str::contains(
        "Expected identifier but got 'StringLiteral'",
    ));

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON,
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Cannot output data outside of JSON in standard JSON mode.",
    ));

    Ok(())
}

#[test]
fn with_runtime() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--bin",
        "--bin-runtime",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Binary:"))
        .stdout(predicate::str::contains("Binary of the runtime part"));

    Ok(())
}

#[test]
fn no_output_flags() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("No output generated"));

    Ok(())
}

#[test]
fn multiple_files() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/caller/Main.sol"),
        crate::common::contract!("solidity/caller/Callable.sol"),
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary:"));

    Ok(())
}

#[test]
fn all_terminal_outputs() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--bin",
        "--bin-runtime",
        "--metadata",
        "--abi",
        "--hashes",
        "--userdoc",
        "--devdoc",
        "--storage-layout",
        "--transient-storage-layout",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Binary:"))
        .stdout(predicate::str::contains("Binary of the runtime part"))
        .stdout(predicate::str::contains("Function signatures:"))
        .stdout(predicate::str::contains("Contract JSON ABI:"))
        .stdout(predicate::str::contains("Developer Documentation:"))
        .stdout(predicate::str::contains("User Documentation:"))
        .stdout(predicate::str::contains("Contract Storage Layout:"))
        .stdout(predicate::str::contains(
            "Contract Transient Storage Layout:",
        ))
        .stdout(predicate::str::contains("Metadata:"));

    Ok(())
}

#[test]
fn combined_terminal_outputs() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--abi",
        "--metadata",
        "--userdoc",
        "--devdoc",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(
        predicate::str::contains("Contract JSON ABI")
            .and(predicate::str::contains("Metadata"))
            .and(predicate::str::contains("User Documentation"))
            .and(predicate::str::contains("Developer Documentation")),
    );

    Ok(())
}

#[test]
fn interface_empty_yul() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/InterfaceEmptyYul.sol"),
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}
