//!
//! CLI tests for IR output flags (--evmla, --ethir, --emit-llvm-ir).
//!

use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn emit_llvm_to_stdout() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--emit-llvm-ir",
        "--bin",
        "--via-ir",
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("Deploy LLVM IR (unoptimized):"))
        .stdout(predicate::str::contains("Deploy LLVM IR:"))
        .stdout(predicate::str::contains("target datalayout"));

    Ok(())
}

#[test]
fn evmla_to_stdout() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--evmla",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("Deploy EVM legacy assembly:"))
        .stdout(predicate::str::contains("PUSH"));

    Ok(())
}

#[test]
fn ethir_to_stdout() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--ethir",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("Deploy Ethereal IR:"))
        .stdout(predicate::str::contains("block_"));

    Ok(())
}

#[test]
fn emit_llvm_with_output_dir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_output")?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--emit-llvm-ir",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    // Check that LLVM IR files were created
    let entries: Vec<_> = std::fs::read_dir(output_directory.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "ll"))
        .collect();

    assert!(!entries.is_empty(), "Expected .ll files to be created");

    Ok(())
}

#[test]
fn evmla_with_output_dir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_output")?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--evmla",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    // Check that EVMLA files were created
    let entries: Vec<_> = std::fs::read_dir(output_directory.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "evmla"))
        .collect();

    assert!(!entries.is_empty(), "Expected .evmla files to be created");

    Ok(())
}

#[test]
fn ethir_with_output_dir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_output")?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--ethir",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    // Check that EthIR files were created
    let entries: Vec<_> = std::fs::read_dir(output_directory.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "ethir"))
        .collect();

    assert!(!entries.is_empty(), "Expected .ethir files to be created");

    Ok(())
}

#[test]
fn multiple_ir_flags_with_output_dir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_output")?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
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

    // Check that all IR files were created
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

#[test]
fn ir_output_standard_json_error() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity.json"),
        "--emit-llvm-ir",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Cannot output data outside of JSON in standard JSON mode.",
    ));

    Ok(())
}

#[test]
fn ir_output_overwrite_protection() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_output")?;

    // First run to create files
    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--emit-llvm-ir",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    // Second run without --overwrite should fail
    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "Refusing to overwrite an existing file",
    ));

    Ok(())
}

#[test]
fn ir_output_overwrite_allowed() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_output")?;

    // First run to create files
    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--emit-llvm-ir",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    // Second run with --overwrite should succeed
    let args_with_overwrite = &[
        crate::common::contract!("solidity/Test.sol"),
        "--emit-llvm-ir",
        "--overwrite",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args_with_overwrite)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    Ok(())
}

#[test]
fn standard_json_llvm_ir_via_ir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_llvm_ir.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"llvmIrUnoptimized\""))
        .stdout(predicate::str::contains("\"llvmIr\""))
        .stdout(predicate::str::contains("define"))
        .stdout(predicate::str::contains("target datalayout"));

    Ok(())
}

#[test]
fn standard_json_evmla_ethir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_evmla_ethir.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"evmla\""))
        .stdout(predicate::str::contains("\"ethir\""))
        .stdout(predicate::str::contains("\"llvmIrUnoptimized\""))
        .stdout(predicate::str::contains("\"llvmIr\""))
        // EVMLA contains instruction names
        .stdout(predicate::str::contains("PUSH"))
        // EthIR contains block labels
        .stdout(predicate::str::contains("block_"));

    Ok(())
}
