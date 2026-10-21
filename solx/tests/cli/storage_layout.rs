//!
//! CLI tests for the eponymous option.
//!

use std::collections::BTreeMap;

use predicates::prelude::*;
use tempfile::TempDir;
use test_case::test_case;

/// The prefixes of the type identifiers that end with the node id of a definition.
const USER_DEFINED_PREFIXES: [&str; 4] = [
    "t_contract(",
    "t_enum(",
    "t_struct(",
    "t_userDefinedValueType(",
];

/// The expected files hold solc's storage layouts verbatim.
#[test_case(
    crate::common::contract!("solidity/SlangStorageLayout.sol"),
    crate::common::storage_layout!("SlangStorageLayout.json")
)]
fn matches_solc(contract: &str, expected: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[contract, "--storage-layout", "--transient-storage-layout"];

    let result = crate::cli::execute_solx(args)?;
    let output = result.success().get_output().stdout.clone();
    let output = String::from_utf8(output)?;

    let expected: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(std::fs::read_to_string(expected)?.as_str())?;
    let expected = expected
        .into_iter()
        .map(|(name, layouts)| Ok((name, normalize(&layouts)?)))
        .collect::<anyhow::Result<BTreeMap<_, _>>>()?;

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
        assert_eq!(lines.next(), Some("Contract Storage Layout:"), "{name}");
        let storage_layout: serde_json::Value =
            serde_json::from_str(lines.next().expect("storage layout JSON"))?;
        assert_eq!(
            lines.next(),
            Some("Contract Transient Storage Layout:"),
            "{name}"
        );
        let transient_storage_layout: serde_json::Value =
            serde_json::from_str(lines.next().expect("transient storage layout JSON"))?;
        let layouts = serde_json::json!({
            "storageLayout": storage_layout,
            "transientStorageLayout": transient_storage_layout,
        });
        actual.insert(name.to_owned(), normalize(&layouts)?);
    }

    assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn too_much_storage() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/StorageOverflow.sol"),
        "--storage-layout",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .failure()
        .stderr(predicate::str::contains(
            "Error: Slang cannot compute the storage layout of `Big`.",
        ))
        .stderr(predicate::str::contains("StorageOverflow.sol:5:1"));

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

#[test]
fn with_transient() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--storage-layout",
        "--transient-storage-layout",
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(
            predicate::str::contains("Contract Storage Layout").and(predicate::str::contains(
                "Contract Transient Storage Layout",
            )),
        );

    Ok(())
}

/// Replaces each `astId` with `0` and each node id that ends a user-defined type identifier with
/// `N`: solx's are Slang node ids, solc's are its AST ids.
fn normalize(layouts: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let json = serde_json::to_string(layouts)?;
    let mut normalized = String::with_capacity(json.len());
    let mut rest = json.as_str();
    while !rest.is_empty() {
        let identifier_start = if let Some(after) = rest.strip_prefix("\"astId\":") {
            normalized.push_str("\"astId\":0");
            Some(after)
        } else if USER_DEFINED_PREFIXES
            .iter()
            .any(|prefix| rest.starts_with(prefix))
        {
            let name_end = rest.find(')').expect("the name is parenthesized") + 1;
            normalized.push_str(&rest[..name_end]);
            normalized.push('N');
            Some(&rest[name_end..])
        } else {
            None
        };
        if let Some(after) = identifier_start {
            rest = after.trim_start_matches(|character: char| character.is_ascii_digit());
        } else {
            let mut characters = rest.chars();
            normalized.extend(characters.next());
            rest = characters.as_str();
        }
    }
    Ok(serde_json::from_str(normalized.as_str())?)
}
