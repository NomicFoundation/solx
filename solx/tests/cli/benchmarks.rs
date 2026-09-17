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

#[cfg(feature = "slang")]
#[test]
fn records_every_pipeline_stage() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--benchmarks",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("Slang_RunStandardJSON"))
        .stdout(predicate::str::contains("Slang_ParseAndBind"))
        .stdout(predicate::str::contains("Slang_SerializeAST:"))
        .stdout(predicate::str::contains("solx_CreateMLIRContext:"))
        .stdout(predicate::str::contains("solx_EmitSol:"))
        .stdout(predicate::str::contains("solx_RunSolPasses:"))
        .stdout(predicate::str::contains("solx_ExtractMLIRObjects:"))
        .stdout(predicate::str::contains("solx_BuildProject"))
        .stdout(predicate::str::contains("solx_Compile"))
        .stdout(predicate::str::contains("solx_Link"))
        .stdout(predicate::str::contains("/InitVerify/"))
        .stdout(predicate::str::contains("/OptimizeVerify/"))
        .stdout(predicate::str::contains("/EmitBytecode/"))
        .stdout(predicate::str::contains(":deploy/CreateMLIRContext/"))
        .stdout(predicate::str::contains(":runtime/CreateMLIRContext/"))
        .stdout(predicate::str::contains(":deploy/ParseMLIR/"))
        .stdout(predicate::str::contains(":runtime/ParseMLIR/"))
        .stdout(predicate::str::contains(":deploy/MLIRToLLVMIR/"))
        .stdout(predicate::str::contains(":runtime/MLIRToLLVMIR/"))
        .stdout(predicate::str::contains(":deploy/WorkerRoundtrip(0)/"))
        .stdout(predicate::str::contains(":runtime/WorkerRoundtrip(0)/"))
        .stdout(predicate::str::contains("us\n"))
        .stdout(predicate::str::contains("ms").not());

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
