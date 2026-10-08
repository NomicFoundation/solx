//!
//! CLI tests for the `debugSymbols` standard JSON output.
//!

use predicates::prelude::*;

#[test]
fn matches_expected() -> anyhow::Result<()> {
    crate::common::setup()?;

    let expected: serde_json::Value = solx_utils::deserialize_from_str(
        std::fs::read_to_string(crate::common::standard_json_output!("debug_symbols.json"))?
            .as_str(),
    )?;

    let result = crate::cli::execute_solx(&[
        "--standard-json",
        crate::common::standard_json!("debug_symbols.json"),
    ])?
    .success();
    let output: serde_json::Value =
        solx_utils::deserialize_from_slice(result.get_output().stdout.as_slice())?;
    let actual = output["sources"]
        .as_object()
        .expect("Always present")
        .iter()
        .map(|(path, source)| (path.clone(), source["debugSymbols"].clone()))
        .collect::<serde_json::Map<_, _>>();
    assert_eq!(serde_json::Value::Object(actual), expected);

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
fn present_on_compile_error() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("debug_symbols_compile_error.json"),
    ];
    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Identifier not found."))
        .stdout(predicate::str::contains("\"debugSymbols\""));

    Ok(())
}

#[test]
fn profiled_under_benchmarks() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("debug_symbols_benchmarks.json"),
    ];
    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("Slang_DebugSymbols:Main.sol"));

    Ok(())
}
