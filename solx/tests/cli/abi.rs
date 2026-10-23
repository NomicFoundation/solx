//!
//! CLI tests for the eponymous option.
//!

use std::collections::BTreeMap;

use predicates::prelude::*;
use test_case::test_case;

/// The expected files hold solc's ABI verbatim.
#[test_case(
    crate::common::standard_json!("slang_test_abi.json"),
    "tests/data/contracts/solidity/SlangTest.sol",
    crate::common::abi!("SlangTest.json")
)]
#[test_case(
    crate::common::standard_json!("slang_abi.json"),
    "tests/data/contracts/solidity/SlangAbi.sol",
    crate::common::abi!("SlangAbi.json")
)]
fn matches_solc(input: &str, source: &str, expected: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &["--standard-json", input];

    let result = crate::cli::execute_solx(args)?;
    let result = result.success();
    let output: serde_json::Value = serde_json::from_slice(result.get_output().stdout.as_slice())?;

    let expected: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(std::fs::read_to_string(expected)?.as_str())?;
    for (name, expected) in expected {
        let actual = &output["contracts"][source][name.as_str()]["abi"];
        assert_eq!(*actual, expected, "{name}");
    }

    Ok(())
}

#[test]
fn interface_with_contract_base() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/InterfaceContractBase.sol"),
        "--abi",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .failure()
        .stderr(predicate::str::contains(
            "Error: Slang cannot compute the ABI of `IDerived`.",
        ))
        .stderr(predicate::str::contains("InterfaceContractBase.sol:9:1"));

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON,
        "--abi",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Cannot output data outside of JSON in standard JSON mode.",
    ));

    Ok(())
}
