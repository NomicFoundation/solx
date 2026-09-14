//!
//! CLI tests for requests the Slang frontend cannot honor.
//!

use predicates::prelude::*;

#[test]
fn command_line_options_are_aggregated() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--bin",
        "--abi",
        "--storage-layout",
        "--base-path",
        ".",
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .failure()
        .stderr(predicate::str::contains(
            "Command line option --abi is not supported in Slang.",
        ))
        .stderr(predicate::str::contains(
            "Command line option --base-path is not supported in Slang.",
        ))
        .stderr(predicate::str::contains(
            "Command line option --storage-layout is not supported in Slang.",
        ));

    Ok(())
}

#[test]
fn via_ir_warns_and_compiles() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT, "--bin", "--via-ir"];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("Binary:"))
        .stderr(predicate::str::contains(
            "Via IR codegen is not honored in Slang",
        ));

    Ok(())
}

#[test]
fn solc_optimizer_settings_warn() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("optimizer_solc_enabled.json"),
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains(
            r#"Standard JSON option \"optimizer.enabled\" is not honored in Slang"#,
        ))
        .stdout(predicate::str::contains("\"severity\":\"warning\""))
        .stdout(predicate::str::contains("\"object\""));

    Ok(())
}

#[test]
fn metadata_literal() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--bin",
        "--metadata-literal",
    ];

    let result = crate::cli::execute_solx(args)?;

    result.failure().stderr(predicate::str::contains(
        "Command line option --metadata-literal is not supported in Slang.",
    ));

    Ok(())
}

#[test]
fn standard_json_settings_and_selections() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_via_ir.json"),
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains(
            "Via IR codegen is not honored in Slang",
        ))
        .stdout(predicate::str::contains(
            r#"Standard JSON output selection \"abi\" is not supported in Slang."#,
        ));

    Ok(())
}

#[test]
fn standard_json_umbrella_selection_is_accepted() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_evm.json"),
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("\"severity\":\"error\"").not())
        .stdout(predicate::str::contains("\"object\""));

    Ok(())
}
