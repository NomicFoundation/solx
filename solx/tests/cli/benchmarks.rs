//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use tempfile::TempDir;
use test_case::test_case;

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

#[test_case(crate::common::TEST_SOLIDITY_CONTRACT, None, 1; "stub creation code is canned")]
#[test_case(crate::common::TEST_SOLIDITY_CONTRACT, Some("--debug-info"), 1; "stub creation code with debug info is canned")]
#[test_case(crate::common::TEST_SOLIDITY_CONTRACT, Some("--emit-mlir=sol"), 2; "mlir output lowers stub creation code")]
#[test_case(crate::common::contract!("solidity/PayableConstructor.sol"), None, 1; "an empty payable constructor is canned")]
#[test_case(crate::common::contract!("solidity/Recursion.sol"), None, 2; "a constructor with parameters is lowered")]
fn records_every_pipeline_stage(
    path: &str,
    flag: Option<&str>,
    lowered_modules: usize,
) -> anyhow::Result<()> {
    crate::common::setup()?;

    let mut args = vec![path, "--benchmarks", "--bin", "--ast-json"];
    args.extend(flag);

    let result = crate::cli::execute_solx(args.as_slice())?;

    result
        .success()
        .stdout(predicate::str::contains("Compiler_ReadInput").count(1))
        .stdout(predicate::str::contains("Slang_RunStandardJSON").count(1))
        .stdout(predicate::str::contains("Slang_ParseAndBind").count(1))
        .stdout(predicate::str::contains("Slang_SerializeAST:").count(1))
        .stdout(predicate::str::contains("Compiler_CreateMLIRContext").count(1))
        .stdout(predicate::str::contains("Compiler_EmitSol:").count(lowered_modules))
        .stdout(
            predicate::str::is_match(r"Compiler_RunSolPasses:\S*\.sol:\w+: \d+us")?
                .count(lowered_modules),
        )
        .stdout(predicate::str::is_match(
            r"Compiler_RunSolPasses:\S*\.sol:\w+/[^:]+: \d+us",
        )?)
        .stdout(predicate::str::contains("/Rest: ").count(lowered_modules))
        .stdout(predicate::str::contains("/Total: ").count(lowered_modules))
        .stdout(predicate::str::contains("Compiler_MLIRToLLVMIR:").count(lowered_modules))
        .stdout(predicate::str::contains("Compiler_BuildProject").count(1))
        .stdout(predicate::str::contains("Compiler_Compile").count(1))
        .stdout(predicate::str::contains("Compiler_Link").count(1))
        .stdout(predicate::str::contains("/InitVerify/").count(2))
        .stdout(predicate::str::contains("/OptimizeVerify/").count(2))
        .stdout(predicate::str::contains("/EmitBytecode/").count(2))
        .stdout(predicate::str::contains(":deploy/ParseBitcode/").count(1))
        .stdout(predicate::str::contains(":runtime/ParseBitcode/").count(1))
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
        .stdout(predicate::str::contains("Compiler_CreateMLIRContext").not());

    Ok(())
}

#[test_case(crate::common::contract!("solidity/SimpleContract.sol"), "SimpleLibrary", "SimpleContract")]
#[test_case(crate::common::contract!("solidity/SimpleContractFirst.sol"), "SimpleContract", "SimpleLibrary")]
fn lists_objects_in_source_order(path: &str, first: &str, second: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[path, "--benchmarks", "--bin"];

    let result = crate::cli::execute_solx(args)?;
    let assert = result.success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone())?;

    let emission = |name: &str| {
        stdout
            .lines()
            .position(|line| {
                line.starts_with("Compiler_EmitSol:")
                    && line.contains(&format!(".sol:{name}_deployed: "))
            })
            .expect("every object's runtime code is emitted")
    };
    assert!(
        emission(first) < emission(second),
        "the benchmarks of {path} come in declaration order, whichever object is scheduled first"
    );

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
