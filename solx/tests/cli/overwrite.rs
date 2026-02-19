//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use tempfile::TempDir;
use test_case::test_case;

#[test_case("--bin")]
#[test_case("--bin-runtime")]
#[test_case("--asm")]
#[cfg_attr(feature = "solc", test_case("--metadata"))]
#[test_case("--ast-json")]
#[cfg_attr(feature = "solc", test_case("--abi"))]
#[cfg_attr(feature = "solc", test_case("--hashes"))]
#[cfg_attr(feature = "solc", test_case("--userdoc"))]
#[cfg_attr(feature = "solc", test_case("--devdoc"))]
#[cfg_attr(feature = "solc", test_case("--storage-layout"))]
#[cfg_attr(feature = "solc", test_case("--transient-storage-layout"))]
#[cfg_attr(feature = "solc", test_case("--asm-solc-json"))]
#[cfg_attr(feature = "solc", test_case("--ir"))]
#[test_case("--benchmarks")]
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
#[cfg_attr(feature = "solc", test_case("--metadata"))]
#[test_case("--ast-json")]
#[cfg_attr(feature = "solc", test_case("--abi"))]
#[cfg_attr(feature = "solc", test_case("--hashes"))]
#[cfg_attr(feature = "solc", test_case("--userdoc"))]
#[cfg_attr(feature = "solc", test_case("--devdoc"))]
#[cfg_attr(feature = "solc", test_case("--storage-layout"))]
#[cfg_attr(feature = "solc", test_case("--transient-storage-layout"))]
#[cfg_attr(feature = "solc", test_case("--asm-solc-json"))]
#[cfg_attr(feature = "solc", test_case("--ir"))]
#[test_case("--benchmarks")]
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
        "Error: Refusing to overwrite an existing file",
    ));
    assert!(output_directory.path().exists());

    Ok(())
}

#[cfg(feature = "solc")]
#[test]
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
        "--asm-solc-json",
        "--ir",
        "--benchmarks",
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

#[cfg(feature = "solc")]
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
        "--asm-solc-json",
        "--ir",
        "--benchmarks",
        "--output-dir",
        output_directory.path().to_str().unwrap(),
    ];

    let _ = crate::cli::execute_solx(args)?;
    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "Error: Refusing to overwrite an existing file",
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
