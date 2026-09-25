//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use test_case::test_case;

#[test]
fn unsupported() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_YUL_CONTRACT, "--yul", "--bin"];

    let result = crate::cli::execute_solx(args)?;
    result.failure().stderr(predicate::str::contains(
        "Slang frontend only supports Solidity sources.",
    ));

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
fn invalid_standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_YUL_CONTRACT, "--yul", "--standard-json"];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Only one mode is allowed at the same time:",
    ));

    Ok(())
}
