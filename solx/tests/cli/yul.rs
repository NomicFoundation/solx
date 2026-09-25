//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use tempfile::TempDir;
use test_case::test_case;

#[test]
fn unsupported() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_YUL_CONTRACT, "--yul", "--bin"];

    let result = crate::cli::execute_solx(args)?;
    result
        .failure()
        .stderr(predicate::str::contains("Yul is not supported yet."));

    Ok(())
}

#[test]
#[ignore = "solx does not lower Yul yet"]
fn bin() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_YUL_CONTRACT, "--yul", "--bin"];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

#[test]
#[ignore = "solx does not lower Yul yet"]
fn stdin() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--yul",
        "--bin",
        solx_standard_json::InputSource::STDIN_INPUT_IDENTIFIER,
    ];

    let result = crate::cli::execute_solx_with_stdin(args, crate::common::TEST_YUL_CONTRACT)?;

    result
        .success()
        .stdout(predicate::str::contains("Binary").count(1));

    Ok(())
}

#[test]
#[ignore = "solx does not lower Yul yet"]
fn asm() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_YUL_CONTRACT, "--yul", "--asm"];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("assembly"));

    Ok(())
}

#[test]
#[ignore = "solx does not lower Yul yet"]
fn metadata() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_YUL_CONTRACT, "--yul", "--metadata"];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Metadata"));

    Ok(())
}

#[test_case("--ast-json")]
#[test_case("--abi")]
#[test_case("--hashes")]
#[test_case("--userdoc")]
#[test_case("--devdoc")]
#[test_case("--storage-layout")]
#[test_case("--transient-storage-layout")]
fn unavailable(flag: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_YUL_CONTRACT, "--yul", flag];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "can be only emitted for Solidity contracts",
    ));

    Ok(())
}

#[test]
#[ignore = "solx does not lower Yul yet"]
fn object_naming() -> anyhow::Result<()> {
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

#[test]
#[ignore = "solx does not lower Yul yet"]
fn solc() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_YUL_CONTRACT, "--yul", "--bin"];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains("Binary"));

    Ok(())
}

#[test]
#[ignore = "solx does not lower Yul yet"]
fn invalid_input() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT, "--yul"];

    let result = crate::cli::execute_solx(args)?;
    result
        .failure()
        .stderr(predicate::str::contains("Yul parsing"));

    Ok(())
}

#[test]
fn invalid_standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_YUL_CONTRACT, "--yul", "--standard-json"];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Only one mode is allowed at the same time:",
    ));

    Ok(())
}

#[test]
#[ignore = "solx does not lower Yul yet"]
fn standard_json_default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &["--standard-json", crate::common::standard_json!("yul.json")];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"))
        .stdout(predicate::str::contains("object"));

    Ok(())
}

#[test]
#[ignore = "solx does not lower Yul yet"]
fn standard_json_default_urls() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("yul_urls.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"))
        .stdout(predicate::str::contains("object"));

    Ok(())
}

#[test]
#[ignore = "solx does not lower Yul yet"]
fn standard_json_default_urls_invalid() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("yul_urls_invalid.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "DeclarationError: Function \\\"mdelete\\\" not found.",
    ));

    Ok(())
}

#[test]
#[ignore = "solx does not lower Yul yet"]
fn standard_json_default_urls_debug_info() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("yul_urls_debug_info.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Error: Debug info is only supported for Solidity source code input.",
    ));

    Ok(())
}

#[test]
#[ignore = "solx does not lower Yul yet"]
fn bin_runtime() -> anyhow::Result<()> {
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

#[test]
#[ignore = "solx does not lower Yul yet"]
fn asm_parser_coverage() -> anyhow::Result<()> {
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

#[test]
#[ignore = "solx does not lower Yul yet"]
fn emit_llvm_ir() -> anyhow::Result<()> {
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

#[test]
#[ignore = "solx does not lower Yul yet"]
fn output_dir() -> anyhow::Result<()> {
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

#[test]
#[ignore = "solx does not lower Yul yet"]
fn output_dir_multiple() -> anyhow::Result<()> {
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

#[test]
#[ignore = "solx does not lower Yul yet"]
fn opcode_coverage_bin() -> anyhow::Result<()> {
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

#[test]
#[ignore = "solx does not lower Yul yet"]
fn opcode_coverage_bin_runtime() -> anyhow::Result<()> {
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

#[test]
#[ignore = "solx does not lower Yul yet"]
fn external_calls_bin() -> anyhow::Result<()> {
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

#[test]
#[ignore = "solx does not lower Yul yet"]
fn external_calls_bin_runtime() -> anyhow::Result<()> {
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

#[test]
fn debug_info_error() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_YUL_CONTRACT, "--yul", "--debug-info"];

    let result = crate::cli::execute_solx(args)?;
    result
        .failure()
        .stderr(predicate::str::contains("can be only emitted for Solidity"));

    Ok(())
}
