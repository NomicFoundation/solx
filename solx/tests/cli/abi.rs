//!
//! CLI tests for the eponymous option.
//!

use std::collections::BTreeMap;

use predicates::prelude::*;
use test_case::test_case;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT, "--abi"];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("Contract JSON ABI").count(1));

    Ok(())
}

/// The expected files hold solc's ABI without `internalType`, which the Slang frontend does not
/// emit yet; both frontends are compared to them the same way.
#[test_case(
    crate::common::contract!("solidity/SlangTest.sol"),
    crate::common::abi!("SlangTest.json")
)]
#[test_case(
    crate::common::contract!("solidity/SlangAbi.sol"),
    crate::common::abi!("SlangAbi.json")
)]
fn matches_solc(contract: &str, expected: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[contract, "--abi"];

    let result = crate::cli::execute_solx(args)?;
    let output = result.success().get_output().stdout.clone();
    let output = String::from_utf8(output)?;

    let expected: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(std::fs::read_to_string(expected)?.as_str())?;

    let mut actual = BTreeMap::new();
    let mut lines = output.lines();
    while let Some(line) = lines.next() {
        let Some(name) = line
            .strip_prefix("======= ")
            .and_then(|line| line.strip_suffix(" ======="))
            .and_then(|full_path| full_path.rsplit(':').next())
        else {
            continue;
        };
        assert_eq!(lines.next(), Some("Contract JSON ABI:"), "{name}");
        let abi: serde_json::Value = serde_json::from_str(lines.next().expect("ABI JSON"))?;
        actual.insert(name.to_owned(), without_internal_type(abi));
    }

    assert_eq!(actual, expected);

    Ok(())
}

/// Removes every `internalType` key, recursively.
fn without_internal_type(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.into_iter().map(without_internal_type).collect())
        }
        serde_json::Value::Object(members) => serde_json::Value::Object(
            members
                .into_iter()
                .filter(|(key, _)| key != "internalType")
                .map(|(key, value)| (key, without_internal_type(value)))
                .collect(),
        ),
        scalar => scalar,
    }
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
