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
        "--metadata",
        "--base-path",
        ".",
        "--ir",
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .failure()
        .stderr(predicate::str::contains(
            "Command line option --metadata is not supported in Slang.",
        ))
        .stderr(predicate::str::contains(
            "Command line option --base-path is not supported in Slang.",
        ))
        .stderr(predicate::str::contains("--ir is ignored in Slang"));

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
        .stderr(predicate::str::contains("--via-ir is ignored in Slang"));

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
            r#"Standard JSON option \"optimizer.enabled\" is ignored in Slang"#,
        ))
        .stdout(predicate::str::contains("\"severity\":\"warning\""))
        .stdout(predicate::str::contains("\"object\""));

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
            r#"Standard JSON option \"viaIR\" is ignored in Slang"#,
        ))
        .stdout(predicate::str::contains(
            r#"Standard JSON output selection \"metadata\" is not supported in Slang."#,
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

#[test]
fn pipeline_output_warns_beside_bytecode() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT, "--bin", "--ir"];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("Binary:"))
        .stderr(predicate::str::contains(
            "--ir is ignored in Slang, which does not have the solc codegen pipelines that produce Yul IR.",
        ));

    Ok(())
}

#[test]
fn nothing_left_to_produce() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT, "--ir", "--evmla"];

    let result = crate::cli::execute_solx(args)?;

    // Warning per request, error because the run would produce nothing at all.
    result
        .failure()
        .stderr(predicate::str::contains("--ir is ignored"))
        .stderr(predicate::str::contains("--evmla is ignored"))
        .stderr(predicate::str::contains(
            "Nothing would be produced: every requested output is unavailable in Slang.",
        ));

    Ok(())
}

#[test]
fn nothing_left_to_produce_in_standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("unsupported_pipeline_selection.json"),
    ];

    let result = crate::cli::execute_solx(args)?;

    result.success().stdout(predicate::str::contains(
        "Nothing would be produced: every requested output is unavailable in Slang.",
    ));

    Ok(())
}

#[test]
fn unsupported_output_alone_does_not_repeat_itself() -> anyhow::Result<()> {
    crate::common::setup()?;

    // An unimplemented output already errors, so the empty-output error stays out of the way.
    let args = &[crate::common::TEST_SOLIDITY_CONTRACT, "--metadata"];

    let result = crate::cli::execute_solx(args)?;

    result
        .failure()
        .stderr(predicate::str::contains(
            "Command line option --metadata is not supported in Slang.",
        ))
        .stderr(predicate::str::contains("Nothing would be produced").not());

    Ok(())
}
