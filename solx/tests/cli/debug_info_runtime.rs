//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;
use test_case::test_case;

#[test_case(true ; "yul")]
#[test_case(false ; "evmla")]
fn default(via_ir: bool) -> anyhow::Result<()> {
    crate::common::setup()?;

    let mut args = vec![
        crate::common::contract!("solidity/Test.sol"),
        "--debug-info-runtime",
    ];
    if via_ir {
        args.push("--via-ir");
    }

    let result = crate::cli::execute_solx(&args)?;

    result
        .success()
        .stdout(predicate::str::contains("Debug info of the runtime part").count(1));

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("solidity.json"),
        "--debug-info-runtime",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Cannot output data outside of JSON in standard JSON mode.",
    ));

    Ok(())
}
