//!
//! CLI tests for the eponymous option.
//!

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT, "--threads", "1"];

    let result = crate::cli::execute_solx(args)?;
    result.success();

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON,
        "--threads",
        "1",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success();

    Ok(())
}
