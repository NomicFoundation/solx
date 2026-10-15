//!
//! CLI tests for the `debugSymbols` standard JSON output.
//!

use predicates::prelude::*;

#[test]
fn matches_expected() -> anyhow::Result<()> {
    crate::common::setup()?;

    let expected: serde_json::Value = solx_utils::deserialize_from_str(
        std::fs::read_to_string(crate::common::debug_symbols!("debug_symbols.json"))?.as_str(),
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

    for symbols in expected.as_object().expect("Always present").values() {
        let parsed: solx_standard_json::output::source::debug_symbols::DebugSymbols =
            serde_json::from_value(symbols.clone())?;
        assert_eq!(&serde_json::to_value(parsed)?, symbols);
    }

    Ok(())
}

#[test]
fn absent_unless_selected() -> anyhow::Result<()> {
    crate::common::setup()?;

    for input in [
        crate::common::standard_json!("select_ast_only.json"),
        crate::common::standard_json!("debug_symbols_contract_level.json"),
    ] {
        let result = crate::cli::execute_solx(&["--standard-json", input])?;
        result
            .success()
            .stdout(predicate::str::contains("\"debugSymbols\"").not());
    }

    Ok(())
}

#[test]
fn absent_on_compile_error() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("debug_symbols_compile_error.json"),
    ];
    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains(
            "Variables cannot be declared in interfaces.",
        ))
        .stdout(predicate::str::contains("\"debugSymbols\"").not());

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
        .stdout(predicate::str::contains("Slang_BuildDebugSymbols:Main.sol"));

    Ok(())
}

#[test]
fn selected_by_wildcard() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_all_wildcard.json"),
    ];
    let result = crate::cli::execute_solx(args)?.success();
    let output: serde_json::Value =
        solx_utils::deserialize_from_slice(result.get_output().stdout.as_slice())?;
    assert!(output["sources"]["A"]["debugSymbols"].is_object());
    assert!(output["sources"]["A"]["ast"].is_object());

    Ok(())
}

#[test]
fn selected_per_file() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("debug_symbols_per_file.json"),
    ];
    let result = crate::cli::execute_solx(args)?.success();
    let output: serde_json::Value =
        solx_utils::deserialize_from_slice(result.get_output().stdout.as_slice())?;
    assert!(output["sources"]["Main.sol"]["debugSymbols"].is_object());
    assert!(output["sources"]["Base.sol"]["debugSymbols"].is_null());

    Ok(())
}
