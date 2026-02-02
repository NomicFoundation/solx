//!
//! CLI tests for the eponymous option.
//!

use tempfile::TempDir;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let tmp_dir_debug = TempDir::with_prefix("debug_output")?;

    let args = &[
        "--bin",
        crate::common::TEST_SOLIDITY_CONTRACT_PATH,
        "--debug-output-dir",
        tmp_dir_debug.path().to_str().unwrap(),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success();

    Ok(())
}

#[test]
fn with_env_var() -> anyhow::Result<()> {
    crate::common::setup()?;

    let tmp_dir_debug = TempDir::with_prefix("debug_output")?;

    let args = &[
        "--bin",
        crate::common::TEST_SOLIDITY_CONTRACT_PATH,
        "--debug-output-dir",
        tmp_dir_debug.path().to_str().unwrap(),
    ];
    let env_vars = vec![(
        "SOLX_DEBUG_OUTPUT_DIR",
        tmp_dir_debug.path().to_string_lossy().to_string(),
    )];

    let result = crate::cli::execute_solx_with_env_vars(args, env_vars)?;
    result.success();

    Ok(())
}

#[test]
fn yul() -> anyhow::Result<()> {
    crate::common::setup()?;

    let tmp_dir_debug = TempDir::with_prefix("debug_output")?;

    let args = &[
        "--bin",
        "--yul",
        crate::common::TEST_YUL_CONTRACT_PATH,
        "--debug-output-dir",
        tmp_dir_debug.path().to_str().unwrap(),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success();

    Ok(())
}

#[test]
fn llvm_ir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let tmp_dir_debug = TempDir::with_prefix("debug_output")?;

    let args = &[
        "--bin",
        "--llvm-ir",
        crate::common::TEST_LLVM_IR_CONTRACT_PATH,
        "--debug-output-dir",
        tmp_dir_debug.path().to_str().unwrap(),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success();

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let tmp_dir_debug = TempDir::with_prefix("debug_output")?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON_PATH,
        "--debug-output-dir",
        tmp_dir_debug.path().to_str().unwrap(),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success();

    Ok(())
}

#[test]
fn standard_json_debug() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_output")?;

    let mut input_json_output_path = output_directory.path().to_path_buf();
    input_json_output_path.push("input.json");

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON_PATH,
    ];
    let env_vars = vec![(
        "SOLX_STANDARD_JSON_DEBUG",
        input_json_output_path.to_string_lossy().to_string(),
    )];

    let result = crate::cli::execute_solx_with_env_vars(args, env_vars)?;
    result.success();

    assert!(output_directory.path().exists());
    assert!(input_json_output_path.exists());

    Ok(())
}
