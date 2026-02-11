//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--optimization-size-fallback",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Binary:\n"));

    Ok(())
}

#[test]
fn with_env_var() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::contract!("solidity/Test.sol"), "--bin"];
    let env_vars = vec![(
        solx_core::SOLX_OPTIMIZATION_SIZE_FALLBACK_ENV,
        "true".to_owned(),
    )];

    let result = crate::cli::execute_solx_with_env_vars(args, env_vars)?;
    result
        .success()
        .stdout(predicate::str::contains("Binary:\n"));

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity.json"),
        "--optimization-size-fallback",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Size optimization fallback must be specified in standard JSON input settings.",
    ));

    Ok(())
}
