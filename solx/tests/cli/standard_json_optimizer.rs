//!
//! CLI tests for standard JSON optimizer settings variations.
//!
//! These tests exercise serde deserialization paths in `solx-standard-json`
//! and LLVM optimizer configuration in `solx-codegen-evm`.
//!

use predicates::prelude::*;
use test_case::test_case;

#[test_case(crate::common::standard_json!("optimizer_mode_1.json"), "1")]
#[test_case(crate::common::standard_json!("optimizer_mode_2.json"), "2")]
#[test_case(crate::common::standard_json!("optimizer_mode_3.json"), "3")]
#[test_case(crate::common::standard_json!("optimizer_mode_s.json"), "s")]
#[test_case(crate::common::standard_json!("optimizer_mode_z.json"), "z")]
fn mode_produces_bytecode(path: &str, _mode: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &["--standard-json", path];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"object\""))
        .stdout(predicate::str::contains("\"errors\"").not());

    Ok(())
}

#[test]
fn default_optimizer_produces_bytecode() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("optimizer_default.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"object\""))
        .stdout(predicate::str::contains("\"errors\"").not());

    Ok(())
}

#[test]
fn solc_enabled_flag_produces_bytecode() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("optimizer_solc_enabled.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"object\""))
        .stdout(predicate::str::contains("\"errors\"").not());

    Ok(())
}

#[test]
fn size_fallback_false_produces_bytecode() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("optimizer_mode_2.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"object\""))
        .stdout(predicate::str::contains("\"errors\"").not());

    Ok(())
}

#[test]
fn size_fallback_true_produces_bytecode() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("optimizer_mode_z.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"object\""))
        .stdout(predicate::str::contains("\"errors\"").not());

    Ok(())
}

#[test_case('1')]
#[test_case('2')]
#[test_case('3')]
#[test_case('s')]
#[test_case('z')]
fn cli_optimization_with_metadata(level: char) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        &format!("-O{level}"),
        "--bin",
        "--metadata",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Binary"))
        .stdout(predicate::str::contains("Metadata"));

    Ok(())
}

#[test_case('1')]
#[test_case('2')]
#[test_case('3')]
#[test_case('s')]
#[test_case('z')]
fn cli_optimization_with_asm(level: char) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        &format!("-O{level}"),
        "--asm",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("assembly"));

    Ok(())
}

#[test_case('s')]
#[test_case('z')]
fn cli_size_optimization_with_size_fallback(level: char) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        &format!("-O{level}"),
        "--optimization-size-fallback",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

#[test_case('s')]
#[test_case('z')]
fn cli_size_optimization_with_size_fallback_env(level: char) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        &format!("-O{level}"),
        "--bin",
    ];
    let env_vars = vec![(
        solx_core::SOLX_OPTIMIZATION_SIZE_FALLBACK_ENV,
        "true".to_owned(),
    )];

    let result = crate::cli::execute_solx_with_env_vars(args, env_vars)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

#[test_case(crate::common::standard_json!("optimizer_mode_1.json"))]
#[test_case(crate::common::standard_json!("optimizer_mode_2.json"))]
#[test_case(crate::common::standard_json!("optimizer_mode_3.json"))]
#[test_case(crate::common::standard_json!("optimizer_mode_s.json"))]
#[test_case(crate::common::standard_json!("optimizer_mode_z.json"))]
#[test_case(crate::common::standard_json!("optimizer_default.json"))]
fn mode_via_stdin(path: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &["--standard-json"];

    let result = crate::cli::execute_solx_with_stdin(args, path)?;
    result
        .success()
        .stdout(predicate::str::contains("\"object\""))
        .stdout(predicate::str::contains("\"errors\"").not());

    Ok(())
}

#[test_case('1')]
#[test_case('2')]
#[test_case('3')]
#[test_case('s')]
#[test_case('z')]
fn cli_all_optimization_levels(level: char) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        &format!("-O{level}"),
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

#[test_case('1')]
#[test_case('2')]
#[test_case('3')]
#[test_case('s')]
#[test_case('z')]
fn cli_optimization_with_bin_runtime(level: char) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        &format!("-O{level}"),
        "--bin-runtime",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}
