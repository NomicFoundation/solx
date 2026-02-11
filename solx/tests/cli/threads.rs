//!
//! CLI tests for the eponymous option.
//!

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/Test.sol"),
        "--threads",
        "1",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success();

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity.json"),
        "--threads",
        "1",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success();

    Ok(())
}
