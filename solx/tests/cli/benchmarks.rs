//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT, "--benchmarks"];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("Benchmarks").count(2));

    Ok(())
}

#[test]
fn records_every_pipeline_stage() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--benchmarks",
        "--bin",
        "--ast-json",
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("Slang_RunStandardJSON").count(1))
        .stdout(predicate::str::contains("Slang_ParseAndBind").count(1))
        .stdout(predicate::str::contains("Slang_SerializeAST:").count(1))
        .stdout(predicate::str::contains("Compiler_CreateMLIRContext:").count(1))
        .stdout(predicate::str::contains("Compiler_EmitSol:").count(1))
        .stdout(predicate::str::is_match(r"Compiler_RunSolPasses:\S*\.sol:\w+: \d+us")?.count(1))
        .stdout(predicate::str::is_match(
            r"Compiler_RunSolPasses:\S*\.sol:\w+/[^:]+: \d+us",
        )?)
        .stdout(predicate::str::contains("/Rest: ").count(1))
        .stdout(predicate::str::contains("/Total: ").count(1))
        .stdout(predicate::str::contains("Compiler_ExtractMLIRObjects:").count(1))
        .stdout(predicate::str::contains("Compiler_BuildProject").count(1))
        .stdout(predicate::str::contains("Compiler_Compile").count(1))
        .stdout(predicate::str::contains("Compiler_Link").count(1))
        .stdout(predicate::str::contains("/InitVerify/").count(2))
        .stdout(predicate::str::contains("/OptimizeVerify/").count(2))
        .stdout(predicate::str::contains("/EmitBytecode/").count(2))
        .stdout(predicate::str::contains("/CreateMLIRContext/").count(1))
        .stdout(predicate::str::contains(":deploy/ParseMLIR/").count(1))
        .stdout(predicate::str::contains(":runtime/ParseMLIR/").count(1))
        .stdout(predicate::str::contains(":deploy/MLIRToLLVMIR/").count(1))
        .stdout(predicate::str::contains(":runtime/MLIRToLLVMIR/").count(1))
        .stdout(predicate::str::contains(":deploy/WorkerRoundtrip(0)/").count(1))
        .stdout(predicate::str::contains(":runtime/WorkerRoundtrip(0)/").count(1))
        .stdout(predicate::str::contains("us\n"))
        .stdout(predicate::str::contains("ms").not());

    Ok(())
}

#[test]
fn creates_no_mlir_context_without_objects() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Abstract.sol"),
        "--benchmarks",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("Slang_ParseAndBind").count(1))
        .stdout(predicate::str::contains("Compiler_CreateMLIRContext:").not());

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON,
        "--benchmarks",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Cannot output data outside of JSON in standard JSON mode.",
    ));

    Ok(())
}

#[test]
fn output_dir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_bench_output")?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--benchmarks",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    Ok(())
}
