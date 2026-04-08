//!
//! CLI tests for the eponymous option.
//!

use std::path::PathBuf;

use predicates::prelude::*;
use tempfile::TempDir;
use test_case::test_case;

#[cfg(feature = "solc")]
#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_output")?;

    let args = &[
        crate::common::contract!("solidity/caller/Main.sol"),
        "--bin",
        "--bin-runtime",
        "--asm",
        "--metadata",
        "--ast-json",
        "--abi",
        "--hashes",
        "--userdoc",
        "--devdoc",
        "--storage-layout",
        "--transient-storage-layout",
        "--asm-solc-json",
        "--ir",
        "--benchmarks",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));
    assert!(output_directory.path().exists());

    Ok(())
}

#[test_case(format!(".{}", solx_utils::EXTENSION_EVM_BINARY))]
#[test_case(format!("_llvm.{}", solx_utils::EXTENSION_EVM_ASSEMBLY))]
#[test_case(format!("_meta.{}", solx_utils::EXTENSION_JSON))]
fn yul(extension: String) -> anyhow::Result<()> {
    crate::common::setup()?;

    let input_path = PathBuf::from(crate::common::TEST_YUL_CONTRACT);
    let output_directory = TempDir::with_prefix("solx_output")?;
    let mut output_file = input_path
        .join("Return")
        .to_string_lossy()
        .replace(['\\', '/', '.'], "_");
    output_file.push_str(extension.as_str());

    let args = &[
        input_path.to_str().expect("Always valid"),
        "--yul",
        "--bin",
        "--bin-runtime",
        "--asm",
        "--metadata",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    assert!(output_directory.path().exists());
    assert!(output_directory.path().join(output_file.as_str()).exists());

    Ok(())
}

#[cfg(feature = "solc")]
#[test]
fn unusual_path_characters() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("File!and#$%-XXXXXX")?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--bin",
        "--bin-runtime",
        "--asm",
        "--metadata",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));
    assert!(output_directory.path().exists());

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON,
        "--output-dir",
        "output",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Output directory cannot be used in standard JSON mode.",
    ));

    Ok(())
}

#[cfg(feature = "solc")]
#[test]
fn multiple_outputs() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_sol_output")?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--bin",
        "--asm",
        "--metadata",
        "--abi",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    Ok(())
}

#[cfg(feature = "solc")]
#[test]
fn multiple_outputs_simple_contract() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_output_dir_multi_test")?;

    let args = &[
        crate::common::contract!("solidity/SimpleContract.sol"),
        "--bin",
        "--abi",
        "--metadata",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success();

    Ok(())
}

#[test]
fn emit_llvm_ir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_output")?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--emit-llvm-ir",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    let entries: Vec<_> = std::fs::read_dir(output_directory.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "ll"))
        .collect();

    assert!(!entries.is_empty(), "Expected .ll files to be created");

    Ok(())
}

#[cfg(feature = "solc")]
#[test]
fn evmla() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_output")?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--evmla",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    let entries: Vec<_> = std::fs::read_dir(output_directory.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "evmla"))
        .collect();

    assert!(!entries.is_empty(), "Expected .evmla files to be created");

    Ok(())
}

#[cfg(feature = "solc")]
#[test]
fn ethir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_output")?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--ethir",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    let entries: Vec<_> = std::fs::read_dir(output_directory.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "ethir"))
        .collect();

    assert!(!entries.is_empty(), "Expected .ethir files to be created");

    Ok(())
}

#[cfg(feature = "mlir")]
#[test]
fn emit_mlir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_output")?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--emit-mlir",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    let entries: Vec<_> = std::fs::read_dir(output_directory.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "mlir"))
        .collect();

    assert!(
        entries.len() >= 2,
        "Expected at least 2 .mlir files (one per dialect stage)"
    );

    let filenames: Vec<_> = entries
        .iter()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    assert!(
        filenames.iter().any(|name| name.contains(".sol.mlir")),
        "Expected a .sol.mlir file, found: {filenames:?}"
    );
    assert!(
        filenames.iter().any(|name| name.contains(".llvm.mlir")),
        "Expected a .llvm.mlir file, found: {filenames:?}"
    );

    Ok(())
}

#[cfg(feature = "solc")]
#[test]
fn multiple_ir_outputs() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_output")?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--emit-llvm-ir",
        "--evmla",
        "--ethir",
        "--asm",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    let ll_files: Vec<_> = std::fs::read_dir(output_directory.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "ll"))
        .collect();

    let evmla_files: Vec<_> = std::fs::read_dir(output_directory.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "evmla"))
        .collect();

    let ethir_files: Vec<_> = std::fs::read_dir(output_directory.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "ethir"))
        .collect();

    let asm_files: Vec<_> = std::fs::read_dir(output_directory.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "asm"))
        .collect();

    assert!(!ll_files.is_empty(), "Expected .ll files to be created");
    assert!(
        !evmla_files.is_empty(),
        "Expected .evmla files to be created"
    );
    assert!(
        !ethir_files.is_empty(),
        "Expected .ethir files to be created"
    );
    assert!(!asm_files.is_empty(), "Expected .asm files to be created");

    Ok(())
}

#[cfg(feature = "mlir")]
#[test]
fn emit_mlir_and_llvm_ir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_output")?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--emit-mlir",
        "--emit-llvm-ir",
        "--asm",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    let mlir_files: Vec<_> = std::fs::read_dir(output_directory.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "mlir"))
        .collect();

    let ll_files: Vec<_> = std::fs::read_dir(output_directory.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "ll"))
        .collect();

    let asm_files: Vec<_> = std::fs::read_dir(output_directory.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "asm"))
        .collect();

    assert!(!mlir_files.is_empty(), "Expected .mlir files to be created");
    assert!(!ll_files.is_empty(), "Expected .ll files to be created");
    assert!(!asm_files.is_empty(), "Expected .asm files to be created");

    Ok(())
}

#[test]
fn env_var() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_debug_env")?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT, "--bin"];

    let result = crate::cli::execute_solx_with_env_vars(
        args,
        vec![(
            "SOLX_OUTPUT_DIR",
            output_directory.path().to_string_lossy().to_string(),
        )],
    )?;

    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}
