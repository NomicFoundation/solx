//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use tempfile::TempDir;
use test_case::test_case;

#[test_case("--bin")]
#[test_case("--bin-runtime")]
#[test_case("--asm")]
#[test_case("--metadata" => ignore["the Slang frontend does not emit this output yet"])]
#[test_case("--ast-json")]
#[test_case("--abi" => ignore["the Slang frontend does not emit this output yet"])]
#[test_case("--hashes" => ignore["the Slang frontend does not emit this output yet"])]
#[test_case("--userdoc" => ignore["the Slang frontend does not emit this output yet"])]
#[test_case("--devdoc" => ignore["the Slang frontend does not emit this output yet"])]
#[test_case("--storage-layout" => ignore["the Slang frontend does not emit this output yet"])]
#[test_case("--transient-storage-layout" => ignore["the Slang frontend does not emit this output yet"])]
#[test_case("--benchmarks")]
#[test_case("--emit-llvm-ir")]
#[test_case("--emit-mlir")]
fn default(flag: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_output")?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        flag,
        "--output-dir",
        output_directory.path().to_str().unwrap(),
        "--overwrite",
    ];

    let _ = crate::cli::execute_solx(args)?;
    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));
    assert!(output_directory.path().exists());

    Ok(())
}

#[test_case("--bin")]
#[test_case("--bin-runtime")]
#[test_case("--asm")]
#[test_case("--metadata" => ignore["the Slang frontend does not emit this output yet"])]
#[test_case("--ast-json")]
#[test_case("--abi" => ignore["the Slang frontend does not emit this output yet"])]
#[test_case("--hashes" => ignore["the Slang frontend does not emit this output yet"])]
#[test_case("--userdoc" => ignore["the Slang frontend does not emit this output yet"])]
#[test_case("--devdoc" => ignore["the Slang frontend does not emit this output yet"])]
#[test_case("--storage-layout" => ignore["the Slang frontend does not emit this output yet"])]
#[test_case("--transient-storage-layout" => ignore["the Slang frontend does not emit this output yet"])]
#[test_case("--benchmarks")]
#[test_case("--emit-llvm-ir")]
#[test_case("--emit-mlir")]
fn missing(flag: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_output")?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        flag,
        "--output-dir",
        output_directory.path().to_str().unwrap(),
    ];

    let _ = crate::cli::execute_solx(args)?;
    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "Refusing to overwrite an existing file",
    ));
    assert!(output_directory.path().exists());

    Ok(())
}

#[test]
#[ignore = "the Slang frontend does not emit this output yet"]
fn all() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_output")?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--bin",
        "--asm",
        "--metadata",
        "--ast-json",
        "--abi",
        "--hashes",
        "--userdoc",
        "--devdoc",
        "--storage-layout",
        "--transient-storage-layout",
        "--benchmarks",
        "--emit-llvm-ir",
        "--output-dir",
        output_directory.path().to_str().unwrap(),
        "--overwrite",
    ];

    let _ = crate::cli::execute_solx(args)?;
    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));
    assert!(output_directory.path().exists());

    Ok(())
}

#[test]
fn all_missing() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_output")?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
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
        "--benchmarks",
        "--emit-llvm-ir",
        "--output-dir",
        output_directory.path().to_str().unwrap(),
    ];

    let _ = crate::cli::execute_solx(args)?;
    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "Refusing to overwrite an existing file",
    ));
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
        "--overwrite",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Overwriting flag cannot be used in standard JSON mode.",
    ));

    Ok(())
}
