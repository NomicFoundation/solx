//!
//! CLI tests for the `debugSymbols` standard JSON output.
//!

use std::collections::BTreeMap;

use predicates::prelude::*;
use solx_standard_json::output::source::debug_symbols::DebugSymbols;

#[test]
fn matches_expected() -> anyhow::Result<()> {
    crate::common::setup()?;

    let expected: BTreeMap<String, DebugSymbols> = solx_utils::deserialize_from_str(
        std::fs::read_to_string("tests/data/standard_json_output/debug_symbols.json")?.as_str(),
    )?;

    let result = crate::cli::execute_solx(&[
        "--standard-json",
        crate::common::standard_json!("debug_symbols.json"),
    ])?
    .success();
    let output: solx_standard_json::Output =
        solx_utils::deserialize_from_slice(result.get_output().stdout.as_slice())?;
    let actual = output
        .sources
        .into_iter()
        .map(|(path, source)| (path, source.debug_symbols.expect("Always selected")))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn absent_unless_selected() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_ast_only.json"),
    ];
    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"ast\""))
        .stdout(predicate::str::contains("\"debugSymbols\"").not());

    Ok(())
}

#[test]
fn selected_by_file_level_wildcard() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_all_wildcard.json"),
    ];
    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("\"debugSymbols\""));

    Ok(())
}
