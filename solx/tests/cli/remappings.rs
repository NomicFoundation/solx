//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "./path/to/1.sol=./path/to/2.sol",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Binary:\n"));

    Ok(())
}

#[test]
fn equals_sign_in_target() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "./path/to/1.sol=./path/to/2.sol=./path/to/3.sol",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Binary:\n"));

    Ok(())
}

#[test]
fn missing_prefix() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "=./path/to/2.sol",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "Remapping `=./path/to/2.sol` prefix is missing.",
    ));

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "./path/to/1.sol=./path/to/2.sol",
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON,
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Input files must be passed via standard JSON input.",
    ));

    Ok(())
}

#[cfg(feature = "slang")]
#[test]
fn resolves_direct_import() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/remapping/Main.sol"),
        crate::common::contract!("solidity/remapping/Dep.sol"),
        "virt/=tests/data/contracts/solidity/remapping/",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Binary:\n").count(1));

    Ok(())
}

#[cfg(feature = "slang")]
#[test]
fn unresolved_import_without_remapping() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/remapping/Main.sol"),
        crate::common::contract!("solidity/remapping/Dep.sol"),
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "Imported file is missing: virt/Dep.sol",
    ));

    Ok(())
}

#[cfg(feature = "slang")]
#[test]
fn standard_json_context_remapping_resolves() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("solidity_with_context_remapping.json"),
    )?;

    result.success().stdout(
        predicate::str::contains("\"npm/dep@1.0.0/Dep.sol\"")
            .and(predicate::str::contains("\"object\"")),
    );

    Ok(())
}

#[cfg(feature = "slang")]
#[test]
fn standard_json_equals_sign_in_target() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("solidity_with_equals_sign_in_remapping_target.json"),
    )?;

    result.success().stdout(
        predicate::str::contains("\"b=c/Dep.sol\"").and(predicate::str::contains("\"object\"")),
    );

    Ok(())
}

#[test]
fn standard_json_invalid_remapping() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("solidity_with_invalid_remapping.json"),
    )?;

    result.success().stdout(predicate::str::contains(
        "Remapping `=missing-prefix/` prefix is missing.",
    ));

    Ok(())
}
