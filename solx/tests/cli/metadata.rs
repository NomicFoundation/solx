//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT, "--metadata"];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("Metadata").count(1));

    Ok(())
}

#[test]
fn invalid_input() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_YUL_CONTRACT, "--metadata"];

    let result = crate::cli::execute_solx(args)?;

    result
        .failure()
        .stderr(predicate::str::contains("Unexpected StringLiteral"));

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON,
        "--metadata",
    ];

    let result = crate::cli::execute_solx(args)?;
    result.success().stdout(predicate::str::contains(
        "Cannot output data outside of JSON in standard JSON mode.",
    ));

    Ok(())
}

#[test]
fn sources_import_closure() -> anyhow::Result<()> {
    crate::common::setup()?;

    let main = crate::common::contract!("solidity/caller/Main.sol");
    let callable = crate::common::contract!("solidity/caller/Callable.sol");
    let unrelated = crate::common::TEST_SOLIDITY_CONTRACT;
    let metadata = metadata(&[main, callable, unrelated, "--metadata"])?;

    assert_eq!(source_names(&metadata["Main"]), [callable, main]);
    assert_eq!(source_names(&metadata["Callable"]), [callable]);

    Ok(())
}

#[test]
fn literal_content() -> anyhow::Result<()> {
    crate::common::setup()?;

    let contract = crate::common::TEST_SOLIDITY_CONTRACT;
    let metadata = metadata(&[contract, "--metadata", "--metadata-literal"])?;
    let source = &metadata["SlangTest"]["sources"][contract];

    assert_eq!(
        source["content"].as_str(),
        Some(std::fs::read_to_string(contract)?.as_str())
    );
    assert!(source.get("urls").is_none());

    Ok(())
}

/// The printed metadata of every contract compiled with `args`, keyed by contract name.
fn metadata(
    args: &[&str],
) -> anyhow::Result<std::collections::BTreeMap<String, serde_json::Value>> {
    let result = crate::cli::execute_solx(args)?;
    let stdout = String::from_utf8(result.success().get_output().stdout.clone())?;

    let mut metadata = std::collections::BTreeMap::new();
    let mut lines = stdout.lines();
    while let Some(line) = lines.next() {
        let Some(header) = line
            .strip_prefix("======= ")
            .and_then(|line| line.strip_suffix(" ======="))
        else {
            continue;
        };
        let (_, name) = header
            .rsplit_once(':')
            .ok_or_else(|| anyhow::anyhow!("no contract name in `{header}`"))?;
        let value = lines
            .by_ref()
            .skip_while(|line| *line != "Metadata:")
            .nth(1)
            .ok_or_else(|| anyhow::anyhow!("no metadata of `{name}` in:\n{stdout}"))?;
        metadata.insert(name.to_owned(), serde_json::from_str(value)?);
    }
    Ok(metadata)
}

/// The names in `metadata`'s `sources`, in their sorted order.
fn source_names(metadata: &serde_json::Value) -> Vec<&str> {
    metadata["sources"]
        .as_object()
        .map(|sources| sources.keys().map(String::as_str).collect())
        .unwrap_or_default()
}
