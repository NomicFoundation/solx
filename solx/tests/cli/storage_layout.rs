//!
//! CLI tests for the eponymous option.
//!

use std::collections::BTreeMap;

use predicates::prelude::*;
use tempfile::TempDir;
use test_case::test_case;

/// The expected files hold solc's storage layouts verbatim.
#[test_case(
    crate::common::standard_json!("slang_storage_layout.json"),
    "tests/data/contracts/solidity/SlangStorageLayout.sol",
    crate::common::storage_layout!("SlangStorageLayout.json")
)]
fn matches_solc(input: &str, source: &str, expected: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &["--standard-json", input];

    let result = crate::cli::execute_solx(args)?;
    let result = result.success();
    let output: serde_json::Value = serde_json::from_slice(result.get_output().stdout.as_slice())?;

    let expected: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(std::fs::read_to_string(expected)?.as_str())?;
    for (name, mut expected) in expected {
        let contract = &output["contracts"][source][name.as_str()];
        let mut actual = serde_json::json!({
            "storageLayout": contract["storageLayout"],
            "transientStorageLayout": contract["transientStorageLayout"],
        });
        renumber(&mut actual, &mut BTreeMap::new());
        renumber(&mut expected, &mut BTreeMap::new());
        assert_eq!(actual, expected, "{name}");
    }

    Ok(())
}

/// Only the outputs that need the storage slots fail.
#[test_case("--storage-layout", true)]
#[test_case("--bin", true)]
#[test_case("--abi", false)]
#[test_case("--hashes", false)]
fn too_much_storage(flag: &str, fails: bool) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/StorageOverflow.sol"),
        flag,
    ];

    let result = crate::cli::execute_solx(args)?;
    if fails {
        result
            .failure()
            .stderr(predicate::str::contains(
                "Error: Slang cannot compute the storage layout of `Big`.",
            ))
            .stderr(predicate::str::contains("StorageOverflow.sol:5:1"));
    } else {
        result.success();
    }

    Ok(())
}

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT, "--storage-layout"];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("Contract Storage Layout").count(1));

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON,
        "--storage-layout",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Cannot output data outside of JSON in standard JSON mode.",
    ));

    Ok(())
}

#[test]
fn output_dir() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output_directory = TempDir::with_prefix("solx_storage_output")?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
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

/// Numbers every node id by first appearance: the `astId` of a variable or member, and the id
/// that closes a definition's type identifier, as a `types` key or a reference. solx's are Slang
/// node ids, solc's are its AST ids.
fn renumber(value: &mut serde_json::Value, ids: &mut BTreeMap<String, usize>) {
    match value {
        serde_json::Value::Object(object) => {
            *object = std::mem::take(object)
                .into_iter()
                .map(|(key, mut value)| {
                    if key == "astId" {
                        value = number(value.to_string(), ids).into();
                    } else {
                        renumber(&mut value, ids);
                    }
                    (renumber_identifier(key.as_str(), ids), value)
                })
                .collect();
        }
        serde_json::Value::Array(array) => array.iter_mut().for_each(|value| renumber(value, ids)),
        serde_json::Value::String(string) => *string = renumber_identifier(string, ids),
        _ => {}
    }
}

/// Numbers the id that closes each definition's type identifier in `text`. The length that closes
/// a fixed-size array stays.
fn renumber_identifier(text: &str, ids: &mut BTreeMap<String, usize>) -> String {
    let definition =
        regex::Regex::new(r"(t_(?:contract|enum|struct|userDefinedValueType)\([^)]*\))(\d+)")
            .expect("the pattern is valid");
    definition
        .replace_all(text, |captures: &regex::Captures| {
            format!("{}{}", &captures[1], number(captures[2].to_owned(), ids))
        })
        .into_owned()
}

/// The number of `id`, assigned on its first appearance.
fn number(id: String, ids: &mut BTreeMap<String, usize>) -> usize {
    let next = ids.len();
    *ids.entry(id).or_insert(next)
}
