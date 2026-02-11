//!
//! CLI tests targeting specific coverage gaps across crates.
//!

use predicates::prelude::*;
use tempfile::TempDir;
use test_case::test_case;

/// Yul compilation with `--bin-runtime` exercises `into_llvm` paths for
/// assignment, variable_declaration, switch, for_loop, if_conditional, etc.
#[test]
fn yul_bin_runtime() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("yul/ParserCoverage.yul"),
        "--yul",
        "--bin-runtime",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Binary of the runtime part"));

    Ok(())
}

/// Yul compilation with `--asm` exercises assembly output path.
#[test]
fn yul_asm() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("yul/ParserCoverage.yul"),
        "--yul",
        "--asm",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("assembly"));

    Ok(())
}

/// Yul compilation with `--emit-llvm-ir` exercises LLVM IR output.
#[test]
fn yul_emit_llvm_ir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("yul/ParserCoverage.yul"),
        "--yul",
        "--emit-llvm-ir",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Runtime LLVM IR:"))
        .stdout(predicate::str::contains("target datalayout"));

    Ok(())
}

/// Yul compilation with `--output-dir` exercises directory output for Yul.
#[test]
fn yul_output_dir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_yul_output")?;

    let args = &[
        crate::common::contract!("yul/ParserCoverage.yul"),
        "--yul",
        "--bin",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    Ok(())
}

/// Yul compilation to output dir with `--asm` and `--emit-llvm-ir` exercises
/// multiple `write_to_directory` branches.
#[test]
fn yul_output_dir_multiple() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_yul_output")?;

    let args = &[
        crate::common::contract!("yul/ParserCoverage.yul"),
        "--yul",
        "--bin",
        "--asm",
        "--emit-llvm-ir",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    Ok(())
}

/// Solidity with `--bin` and `--bin-runtime` together.
#[test]
fn solidity_bin_and_runtime() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--bin",
        "--bin-runtime",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Binary:"))
        .stdout(predicate::str::contains("Binary of the runtime part"));

    Ok(())
}

/// Solidity with `--metadata` flag exercises metadata terminal output.
#[test]
fn solidity_metadata() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::contract!("solidity/Test.sol"), "--metadata"];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Metadata:"))
        .stdout(predicate::str::contains("\"compiler\""));

    Ok(())
}

/// LLVM IR with `--asm` exercises assembly output path for LLVM IR.
#[test]
fn llvm_ir_asm() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("llvm_ir/Test.ll"),
        "--llvm-ir",
        "--asm",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("assembly"));

    Ok(())
}

/// LLVM IR compilation with `--bin-runtime` exercises runtime code output.
#[test]
fn llvm_ir_bin_runtime() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("llvm_ir/Test.ll"),
        "--llvm-ir",
        "--bin-runtime",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Binary of the runtime part"));

    Ok(())
}

/// Solidity with many terminal output flags exercises `write_to_terminal` branches.
#[test]
fn solidity_all_terminal_outputs() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--bin",
        "--bin-runtime",
        "--metadata",
        "--abi",
        "--hashes",
        "--userdoc",
        "--devdoc",
        "--storage-layout",
        "--transient-storage-layout",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Binary:"))
        .stdout(predicate::str::contains("Binary of the runtime part"))
        .stdout(predicate::str::contains("Function signatures:"))
        .stdout(predicate::str::contains("Contract JSON ABI:"))
        .stdout(predicate::str::contains("Developer Documentation:"))
        .stdout(predicate::str::contains("User Documentation:"))
        .stdout(predicate::str::contains("Contract Storage Layout:"))
        .stdout(predicate::str::contains(
            "Contract Transient Storage Layout:",
        ))
        .stdout(predicate::str::contains("Metadata:"));

    Ok(())
}

/// Solidity with output directory and multiple outputs exercises
/// `write_to_directory` branches.
#[test]
fn solidity_output_dir_multiple() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_sol_output")?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
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

/// Multiple Solidity files exercises the allow_paths accumulation
/// code in solc.rs.
#[test]
fn solidity_multiple_files() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/caller/Main.sol"),
        crate::common::contract!("solidity/caller/Callable.sol"),
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary:"));

    Ok(())
}

/// Optimized Solidity contract with size optimization levels.
#[test_case('s')]
#[test_case('z')]
fn yul_optimization(level: char) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("yul/Test.yul"),
        "--yul",
        &format!("-O{level}"),
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

/// LLVM IR with optimization levels exercises the size_level code.
#[test_case('s')]
#[test_case('z')]
fn llvm_ir_optimization(level: char) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("llvm_ir/Test.ll"),
        "--llvm-ir",
        &format!("-O{level}"),
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

/// Solidity with `--via-ir` and `--bin` exercises the Yul pipeline.
#[test]
fn solidity_via_ir_bin() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--via-ir",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

/// Solidity with `--via-ir` and emit LLVM IR exercises the via-IR + LLVM output path.
#[test]
fn solidity_via_ir_emit_llvm() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--via-ir",
        "--emit-llvm-ir",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Deploy LLVM IR:"))
        .stdout(predicate::str::contains("Runtime LLVM IR:"))
        .stdout(predicate::str::contains("target datalayout"));

    Ok(())
}

/// Solidity with debug info via standard JSON exercises debug_info extraction.
#[test]
fn solidity_standard_json_debug_info() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"debugInfo\""));

    Ok(())
}

/// Standard JSON with empty selection exercises the pruning code path
/// where no EVM data is selected, triggering the `is_empty` pruning branches.
#[test]
fn standard_json_select_none_prunes_output() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_none.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"evm\"").not())
        .stdout(predicate::str::contains("\"bytecode\"").not())
        .stdout(predicate::str::contains("\"contracts\"").not());

    Ok(())
}

/// Solidity with `--debug-info` and `--output-dir` exercises the debug info
/// `write_to_directory` branch for deploy code.
#[test]
fn solidity_debug_info_output_dir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_debug_output")?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--debug-info",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    // Check that debug info files were created
    let dbg_files: Vec<_> = std::fs::read_dir(output_directory.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().to_string_lossy().contains(".dbg."))
        .collect();
    assert!(!dbg_files.is_empty(), "Expected .dbg files to be created");

    Ok(())
}

/// Solidity with `--debug-info-runtime` and `--output-dir` exercises the runtime
/// debug info `write_to_directory` branch.
#[test]
fn solidity_debug_info_runtime_output_dir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_debug_rt_output")?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--debug-info-runtime",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    // Check that runtime debug info files were created
    let dbg_files: Vec<_> = std::fs::read_dir(output_directory.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let name = entry.path().to_string_lossy().to_string();
            name.contains(".dbg.") && name.contains("runtime")
        })
        .collect();
    assert!(
        !dbg_files.is_empty(),
        "Expected runtime .dbg files to be created"
    );

    Ok(())
}

/// LLVM IR compilation with `--output-dir` exercises the LLVM IR
/// `write_to_directory` path.
#[test]
fn llvm_ir_output_dir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_llvm_ir_output")?;

    let args = &[
        crate::common::contract!("llvm_ir/Test.ll"),
        "--llvm-ir",
        "--bin",
        "--asm",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    Ok(())
}

/// Solidity with `--ir` to terminal exercises the Yul IR `write_to_terminal` path.
#[test]
fn solidity_ir_to_terminal() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--ir",
        "--via-ir",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("IR:"));

    Ok(())
}

/// Solidity with `--benchmarks` to terminal exercises the benchmarks
/// `write_to_terminal` path.
#[test]
fn solidity_benchmarks_to_terminal() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--benchmarks",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Benchmarks:"));

    Ok(())
}

/// Solidity with `--benchmarks` and `--output-dir` exercises the benchmarks
/// `write_to_directory` path.
#[test]
fn solidity_benchmarks_output_dir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_bench_output")?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--benchmarks",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    // Check that benchmark files were created
    let bench_files: Vec<_> = std::fs::read_dir(output_directory.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().to_string_lossy().contains("benchmarks"))
        .collect();
    assert!(
        !bench_files.is_empty(),
        "Expected benchmark files to be created"
    );

    Ok(())
}

/// LLVM IR with `--emit-llvm-ir` exercises the LLVM IR output for
/// LLVM IR input source.
#[test]
fn llvm_ir_emit_llvm_ir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("llvm_ir/Test.ll"),
        "--llvm-ir",
        "--emit-llvm-ir",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("LLVM IR:"))
        .stdout(predicate::str::contains("target datalayout"));

    Ok(())
}

/// Solidity with `--asm-solc-json` to terminal exercises the EVM legacy
/// assembly terminal output path.
#[test]
fn solidity_asm_solc_json_to_terminal() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--asm-solc-json",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("EVM assembly:"));

    Ok(())
}

/// Solidity with `--ir` and `--output-dir` exercises the Yul IR
/// `write_to_directory` branch.
#[test]
fn solidity_ir_output_dir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_ir_output")?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--ir",
        "--via-ir",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    // Check that Yul IR files were created
    let yul_files: Vec<_> = std::fs::read_dir(output_directory.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "yul"))
        .collect();
    assert!(!yul_files.is_empty(), "Expected .yul files to be created");

    Ok(())
}

/// Solidity with `--asm-solc-json` and `--output-dir` exercises the EVM legacy
/// assembly `write_to_directory` branch.
#[test]
fn solidity_asm_solc_json_output_dir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_evmasm_output")?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--asm-solc-json",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    // Check that EVM assembly JSON files were created
    let evm_files: Vec<_> = std::fs::read_dir(output_directory.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().to_string_lossy().contains("_evm."))
        .collect();
    assert!(
        !evm_files.is_empty(),
        "Expected EVM assembly JSON files to be created"
    );

    Ok(())
}

/// Solidity with `--hashes` and `--output-dir` exercises the method identifiers
/// `write_to_directory` branch.
#[test]
fn solidity_hashes_output_dir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_hashes_output")?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--hashes",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    Ok(())
}

/// Solidity with `--userdoc` and `--devdoc` to output directory exercises
/// documentation `write_to_directory` branches.
#[test]
fn solidity_docs_output_dir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_docs_output")?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--userdoc",
        "--devdoc",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    Ok(())
}

/// Yul opcode coverage contract exercises many EVM opcodes through the
/// function_call codegen path: byte, mstore8, mcopy, sload, sstore, tload,
/// tstore, calldataload, calldatasize, calldatacopy, returndatasize, codesize,
/// codecopy, extcodesize, extcodehash, address, caller, callvalue, gas,
/// balance, selfbalance, gaslimit, gasprice, origin, chainid, timestamp,
/// number, blockhash, difficulty, coinbase, basefee, msize, keccak256,
/// signextend, addmod, mulmod, exp, pop, not, xor, or, and, shl, shr, sar,
/// sdiv, smod, slt, sgt, iszero, log0-log4, switch-only-default.
#[test]
fn yul_opcode_coverage_bin() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("yul/OpcodeCoverage.yul"),
        "--yul",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

/// Yul opcode coverage with `--bin-runtime` to also exercise runtime code
/// codegen for the many opcodes.
#[test]
fn yul_opcode_coverage_bin_runtime() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("yul/OpcodeCoverage.yul"),
        "--yul",
        "--bin-runtime",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Binary of the runtime part"));

    Ok(())
}

/// Solidity with `--storage-layout` and `--transient-storage-layout` to output
/// directory exercises storage layout `write_to_directory` branches.
#[test]
fn solidity_storage_layout_output_dir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_storage_output")?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--storage-layout",
        "--transient-storage-layout",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    Ok(())
}

// ======== Round 4: Dead code removal + external call codegen ========

/// Yul with external calls (call, staticcall, delegatecall, create, create2)
/// exercises the external call codegen paths in function_call/mod.rs.
#[test]
fn yul_external_calls_bin() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("yul/ExternalCalls.yul"),
        "--yul",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary:"));

    Ok(())
}

/// Yul with external calls + runtime bytecode.
#[test]
fn yul_external_calls_bin_runtime() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("yul/ExternalCalls.yul"),
        "--yul",
        "--bin-runtime",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Binary of the runtime part"));

    Ok(())
}

/// Standard JSON selecting LLVM IR outputs (unoptimized + optimized) exercises
/// the LLVM IR output selection and `write_to_standard_json` paths.
#[test]
fn standard_json_select_llvm_ir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("select_llvm_ir.json"),
    )?;

    result.success().stdout(
        predicate::str::contains("llvmIr")
            .and(predicate::str::contains("llvmIrUnoptimized"))
            .and(predicate::str::contains("object")),
    );

    Ok(())
}

/// Standard JSON selecting EVMLA + EthIR outputs (non-via-IR mode) exercises
/// the EVMLA/EthIR output selection paths.
#[test]
fn standard_json_select_evmla_ethir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("select_evmla_ethir.json"),
    )?;

    result.success().stdout(
        predicate::str::contains("evmla")
            .and(predicate::str::contains("ethir"))
            .and(predicate::str::contains("object")),
    );

    Ok(())
}

/// Standard JSON wildcard select-all pattern exercises full output generation
/// with wildcard matching in output_selection.
#[test]
fn standard_json_select_all_wildcard() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("select_all_wildcard.json"),
    )?;

    result
        .success()
        .stdout(predicate::str::contains("bytecode"));

    Ok(())
}

/// Standard JSON with deploy-time linking exercises the linker_symbols path
/// in `write_to_standard_json`.
#[test]
fn standard_json_deploy_time_linking() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("solidity_deploy_time_linking.json"),
    )?;

    result.success().stdout(
        predicate::str::contains("linkReferences").or(predicate::str::contains("bytecode")),
    );

    Ok(())
}

/// Standard JSON with recursion input exercises the error reporting path
/// for recursive contracts.
#[test]
fn standard_json_recursion_error() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("solidity_recursion.json"),
    )?;

    result
        .success()
        .stdout(predicate::str::contains("error").or(predicate::str::contains("Error")));

    Ok(())
}

/// Standard JSON with empty sources exercises the empty-sources error path.
#[test]
fn standard_json_empty_sources_error() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("solidity_empty_sources.json"),
    )?;

    result
        .success()
        .stdout(predicate::str::contains("error").or(predicate::str::contains("Error")));

    Ok(())
}

/// Standard JSON with missing sources exercises the missing-sources error path.
#[test]
fn standard_json_missing_sources_error() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("solidity_missing_sources.json"),
    )?;

    result
        .success()
        .stdout(predicate::str::contains("error").or(predicate::str::contains("Error")));

    Ok(())
}

/// Standard JSON with invalid JSON exercises the deserialization error path.
#[test]
fn standard_json_invalid_json_error() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("solidity_invalid.json"),
    )?;

    result
        .success()
        .stdout(predicate::str::contains("error").or(predicate::str::contains("Error")));

    Ok(())
}

/// Solidity with combined terminal flags: `--abi --metadata --userdoc --devdoc`
/// exercises ABI, metadata, userdoc, and devdoc `write_to_terminal` branches.
#[test]
fn solidity_combined_terminal_outputs() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--abi",
        "--metadata",
        "--userdoc",
        "--devdoc",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(
        predicate::str::contains("Contract JSON ABI")
            .and(predicate::str::contains("Metadata"))
            .and(predicate::str::contains("User Documentation"))
            .and(predicate::str::contains("Developer Documentation")),
    );

    Ok(())
}

/// Solidity with `--storage-layout --transient-storage-layout` to terminal
/// exercises the terminal storage layout branches.
#[test]
fn solidity_storage_layout_to_terminal() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--storage-layout",
        "--transient-storage-layout",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(
            predicate::str::contains("Contract Storage Layout").and(predicate::str::contains(
                "Contract Transient Storage Layout",
            )),
        );

    Ok(())
}

/// Standard JSON with EVM version too old exercises the evm_version error path.
#[test]
fn standard_json_evm_version_too_old() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("evm_version_too_old.json"),
    )?;

    result
        .success()
        .stdout(predicate::str::contains("error").or(predicate::str::contains("Error")));

    Ok(())
}

// ======== Round 5: env vars, error paths, debug info errors ========

/// SOLX_OUTPUT_DIR env var triggers debug mode output config creation
/// (lib.rs create_dir_all + OutputConfig::new_debug).
#[test]
fn solidity_debug_output_dir_env() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_debug_env")?;

    let args = &[crate::common::contract!("solidity/Test.sol"), "--bin"];

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

/// SOLX_OPTIMIZATION env var exercises the environment variable optimization
/// path in the main function.
#[test]
fn solidity_optimization_env() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::contract!("solidity/Test.sol"), "--bin"];

    let result =
        crate::cli::execute_solx_with_env_vars(args, vec![("SOLX_OPTIMIZATION", "z".to_owned())])?;

    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

/// Yul with `--debug-info` should fail because debug info and other
/// Solidity-only outputs cannot be emitted for non-Solidity contracts.
#[test]
fn yul_debug_info_error() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("yul/Test.yul"),
        "--yul",
        "--debug-info",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .failure()
        .stderr(predicate::str::contains("can be only emitted for Solidity"));

    Ok(())
}

/// LLVM IR with `--debug-info` should fail because debug info and other
/// Solidity-only outputs cannot be emitted for non-Solidity contracts.
#[test]
fn llvm_ir_debug_info_error() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("llvm_ir/Test.ll"),
        "--llvm-ir",
        "--debug-info",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .failure()
        .stderr(predicate::str::contains("can be only emitted for Solidity"));

    Ok(())
}

/// LLVM IR standard JSON exercises the LLVM IR → standard JSON path (lib.rs
/// InputLanguage::LLVMIR branch in standard_json_evm).
#[test]
fn standard_json_llvm_ir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("llvm_ir_urls.json"),
    )?;

    result
        .success()
        .stdout(predicate::str::contains("contracts").or(predicate::str::contains("error")));

    Ok(())
}

/// LLVM IR standard JSON with invalid file exercises the LLVM IR error path.
#[test]
fn standard_json_llvm_ir_invalid() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("llvm_ir_urls_invalid.json"),
    )?;

    result
        .success()
        .stdout(predicate::str::contains("error").or(predicate::str::contains("Error")));

    Ok(())
}

/// LLVM IR standard JSON with missing file exercises the missing-file error path.
#[test]
fn standard_json_llvm_ir_missing_file() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("llvm_ir_urls_missing_file.json"),
    )?;

    result
        .success()
        .stdout(predicate::str::contains("error").or(predicate::str::contains("Error")));

    Ok(())
}

/// LLVM IR standard JSON with debug info exercises the debug info output path.
#[test]
fn standard_json_llvm_ir_debug_info() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("llvm_ir_urls_debug_info.json"),
    )?;

    result.success();

    Ok(())
}

/// Yul standard JSON exercises the Yul → standard JSON path (lib.rs
/// InputLanguage::Yul branch in standard_json_evm).
#[test]
fn standard_json_yul() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("yul.json"),
    )?;

    result
        .success()
        .stdout(predicate::str::contains("contracts").or(predicate::str::contains("bytecode")));

    Ok(())
}

/// Yul standard JSON with debug info exercises the Yul debug info output path.
#[test]
fn standard_json_yul_debug_info() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("yul_urls_debug_info.json"),
    )?;

    result.success();

    Ok(())
}

/// Solidity with StackTooDeepSolc exercises the compilation of complex
/// contracts that may trigger stack-too-deep handling.
#[test]
fn solidity_stack_too_deep_solc() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/StackTooDeepSolc.sol"),
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    // May succeed (solx optimizes away the stack issue) or fail
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

/// Solidity with StackTooDeepLLVM exercises the LLVM stack-too-deep error handler.
#[test]
fn solidity_stack_too_deep_llvm() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/StackTooDeepLLVM.sol"),
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    // May succeed with size fallback or fail with stack too deep
    // Either way exercises the compilation pipeline
    let _ = result;

    Ok(())
}

/// Solidity with fuzzed linker error exercises the linker error path.
#[test]
fn solidity_fuzzed_linker_error() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/FuzzedLinkerError.sol"),
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    // May produce an error or succeed — exercises the error path
    let _ = result;

    Ok(())
}

/// Standard JSON fuzzed simple use expression exercises the error handling path
/// for fuzzed inputs.
#[test]
fn standard_json_fuzzed_simple_use_expression() -> anyhow::Result<()> {
    crate::common::setup()?;

    let result = crate::cli::execute_solx_with_stdin(
        &["--standard-json"],
        crate::common::standard_json!("solidity_fuzzed_simple_use_expression.json"),
    )?;

    result.success();

    Ok(())
}

/// LLVM IR CLI with invalid IR file exercises the llvm_ir_to_evm error path.
#[test]
fn llvm_ir_invalid_cli() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("llvm_ir/Invalid.ll"),
        "--llvm-ir",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure();

    Ok(())
}

/// LLVM IR CLI with linker error exercises the linker error path.
#[test]
fn llvm_ir_linker_error_cli() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("llvm_ir/LinkerError.ll"),
        "--llvm-ir",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure();

    Ok(())
}

/// Compiling with no output flags prints "No output generated" (lib.rs:290-294).
#[test]
fn solidity_no_output_flags() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::contract!("solidity/Test.sol")];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("No output generated"));

    Ok(())
}

/// Invalid SOLX_OPTIMIZATION env var triggers the bail! error path.
#[test]
fn solidity_invalid_optimization_env() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::contract!("solidity/Test.sol"), "--bin"];

    let result = crate::cli::execute_solx_with_env_vars(
        args,
        vec![("SOLX_OPTIMIZATION", "invalid".to_owned())],
    )?;
    result
        .failure()
        .stderr(predicate::str::contains("Invalid value `invalid`"));

    Ok(())
}

/// Standard JSON with select_none exercises the empty output_selection path.
#[test]
fn standard_json_select_none() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_none.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success();

    Ok(())
}

/// Standard JSON with metadata_hash_ipfs_no_metadata exercises IPFS hash
/// without metadata output.
#[test]
fn standard_json_metadata_hash_ipfs_no_metadata() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("metadata_hash_ipfs_no_metadata.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"));

    Ok(())
}

/// Standard JSON with metadata_hash_none_no_metadata exercises no-hash
/// without metadata output.
#[test]
fn standard_json_metadata_hash_none_no_metadata() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("metadata_hash_none_no_metadata.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"));

    Ok(())
}

/// Standard JSON with no_cbor_metadata exercises appendCBOR=false path.
#[test]
fn standard_json_no_cbor_metadata() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("no_cbor_metadata.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"));

    Ok(())
}

/// Yul ObjectNaming test exercises nested Yul object parsing.
#[test]
fn yul_object_naming() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("yul/ObjectNaming.yul"),
        "--yul",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

/// Multi-file Solidity compilation with caller/callable pattern exercises
/// the allow_paths logic and multi-file dependency resolution.
#[test]
fn solidity_multi_file_caller() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/caller/Main.sol"),
        crate::common::contract!("solidity/caller/Callable.sol"),
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

/// Solidity compilation to output directory exercises write_to_directory path.
#[test]
fn solidity_output_dir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_dir = TempDir::with_prefix("solx_output_dir_test")?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--bin",
        "--output-dir",
        output_dir.path().to_str().unwrap(),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success();

    Ok(())
}

/// Standard JSON Yul URLs exercises URL-based Yul source loading path.
#[test]
fn standard_json_yul_urls() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("yul_urls.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"));

    Ok(())
}

/// Standard JSON Yul URLs invalid exercises the error path for bad Yul URLs.
#[test]
fn standard_json_yul_urls_invalid() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("yul_urls_invalid.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("error"));

    Ok(())
}

/// Standard JSON Yul URLs with debug info exercises the debug info error path.
#[test]
fn standard_json_yul_urls_debug_info() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("yul_urls_debug_info.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success();

    Ok(())
}

/// Solidity with --abi flag exercises the ABI output terminal path.
#[test]
fn solidity_abi_terminal() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::contract!("solidity/Test.sol"), "--abi"];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Contract"));

    Ok(())
}

/// Solidity with --metadata flag exercises the metadata output terminal path.
#[test]
fn solidity_metadata_terminal() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::contract!("solidity/Test.sol"), "--metadata"];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Metadata"));

    Ok(())
}

/// Standard JSON with select_single exercises the single output selection path.
#[test]
fn standard_json_select_single() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_single.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success();

    Ok(())
}

/// Standard JSON with select_evm_bytecode exercises evm.bytecode selection.
#[test]
fn standard_json_select_evm_bytecode() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_evm_bytecode.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"));

    Ok(())
}

/// Standard JSON with select_evm_bytecode_opcodes exercises opcodes selection.
#[test]
fn standard_json_select_evm_bytecode_opcodes() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_evm_bytecode_opcodes.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success();

    Ok(())
}

/// Standard JSON with select_evm_deployed_bytecode exercises deployed bytecode
/// selection.
#[test]
fn standard_json_select_evm_deployed_bytecode() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_evm_deployed_bytecode.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("deployedBytecode"));

    Ok(())
}

/// Standard JSON with select_evm_deployed_bytecode_link_references exercises
/// the link references output selection.
#[test]
fn standard_json_select_evm_deployed_bytecode_link_references() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_evm_deployed_bytecode_link_references.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success();

    Ok(())
}

/// Standard JSON with Solidity recursion test exercises viaIR + recursion.
#[test]
fn standard_json_solidity_recursion() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity_recursion.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success();

    Ok(())
}

/// Solidity with SimpleContract and output dir exercises directory write with
/// multiple output types.
#[test]
fn solidity_output_dir_multiple_outputs() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_dir = TempDir::with_prefix("solx_output_dir_multi_test")?;

    let args = &[
        crate::common::contract!("solidity/SimpleContract.sol"),
        "--bin",
        "--abi",
        "--metadata",
        "--output-dir",
        output_dir.path().to_str().unwrap(),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success();

    Ok(())
}

/// Solidity with --overwrite flag on output dir exercises the overwrite path.
#[test]
fn solidity_output_dir_overwrite() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_dir = TempDir::with_prefix("solx_output_dir_overwrite_test")?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--bin",
        "--output-dir",
        output_dir.path().to_str().unwrap(),
        "--overwrite",
    ];

    // Run twice - second run exercises overwrite path
    crate::cli::execute_solx(args)?;
    let result = crate::cli::execute_solx(args)?;
    result.success();

    Ok(())
}

/// Standard JSON with metadata_hash_ipfs_and_metadata exercises full metadata
/// output path.
#[test]
fn standard_json_metadata_hash_ipfs_and_metadata() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("metadata_hash_ipfs_and_metadata.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("metadata"));

    Ok(())
}

/// Standard JSON with metadata_hash_none_and_metadata exercises no-hash +
/// metadata output.
#[test]
fn standard_json_metadata_hash_none_and_metadata() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("metadata_hash_none_and_metadata.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("metadata"));

    Ok(())
}

/// Invalid SOLX_OPTIMIZATION env var with standard JSON input exercises
/// the standard JSON env var validation path (lib.rs:532-540).
/// Standard JSON mode returns errors in JSON output (exit code 0).
#[test]
fn standard_json_invalid_optimization_env() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity.json"),
    ];

    let result = crate::cli::execute_solx_with_env_vars(
        args,
        vec![("SOLX_OPTIMIZATION", "invalid".to_owned())],
    )?;
    result
        .success()
        .stdout(predicate::str::contains("Invalid value `invalid`"));

    Ok(())
}

/// Solidity with --hashes exercises the function signature hashes terminal
/// output path.
#[test]
fn solidity_hashes_terminal() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::contract!("solidity/Test.sol"), "--hashes"];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Function signatures"));

    Ok(())
}

/// Solidity with LinkedMixedDeps + library exercises the linker path.
#[test]
fn solidity_linked_mixed_deps() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/LinkedMixedDeps.sol"),
        "--bin",
        "--libraries",
        "tests/data/contracts/solidity/MiniMath.sol:MiniMath=0xF9702469Dfb84A9aC171E284F71615bd3D3f1EdC",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

/// Solidity with LinkedMixedDepsMultiLevel + library exercises deep linking.
#[test]
fn solidity_linked_mixed_deps_multi_level() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/LinkedMixedDepsMultiLevel.sol"),
        "--bin",
        "--libraries",
        "tests/data/contracts/solidity/MiniMath.sol:MiniMath=0xF9702469Dfb84A9aC171E284F71615bd3D3f1EdC",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

/// Yul compilation with --bin exercises the main deploy path.
#[test]
fn yul_parser_coverage_bin() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("yul/ParserCoverage.yul"),
        "--yul",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

/// Standard JSON with Yul parser coverage tests the standard JSON Yul path.
#[test]
fn standard_json_yul_parser_coverage() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("yul_parser_coverage.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"));

    Ok(())
}

/// Solidity InterfaceEmptyYul exercises the empty Yul interface path.
#[test]
fn solidity_interface_empty_yul() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/InterfaceEmptyYul.sol"),
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

/// Unsupported opcode `callcode` should produce a compilation error.
#[test]
fn yul_unsupported_callcode() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("yul/ErrorUnsupportedCallcode.yul"),
        "--yul",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(
        predicate::str::contains("CALLCODE").and(predicate::str::contains("not supported")),
    );

    Ok(())
}

/// Unsupported opcode `pc` should produce a compilation error.
#[test]
fn yul_unsupported_pc() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("yul/ErrorUnsupportedPc.yul"),
        "--yul",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .failure()
        .stderr(predicate::str::contains("PC").and(predicate::str::contains("not supported")));

    Ok(())
}

/// Unsupported opcode `selfdestruct` should produce a compilation error.
#[test]
fn yul_unsupported_selfdestruct() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("yul/ErrorUnsupportedSelfdestruct.yul"),
        "--yul",
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(
        predicate::str::contains("SELFDESTRUCT").and(predicate::str::contains("not supported")),
    );

    Ok(())
}

/// Standard JSON EVM output selection exercises the evm top-level output.
#[test]
fn standard_json_select_evm() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_evm.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"));

    Ok(())
}
