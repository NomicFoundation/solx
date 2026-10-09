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
    let result = result
        .success()
        .stdout(predicate::str::contains("Identifier not found."));
    let output: serde_json::Value =
        solx_utils::deserialize_from_slice(result.get_output().stdout.as_slice())?;
    assert_eq!(
        output["sources"]["Main.sol"]["debugSymbols"]["contracts"][0]["bases"],
        serde_json::json!([{ "path": "Main.sol", "name": "C" }]),
    );

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
        crate::common::standard_json!("debug_symbols_wildcard.json"),
    ];
    let result = crate::cli::execute_solx(args)?.success();
    let output: serde_json::Value =
        solx_utils::deserialize_from_slice(result.get_output().stdout.as_slice())?;
    for path in ["Base.sol", "Main.sol"] {
        assert!(
            output["sources"][path]["debugSymbols"].is_object(),
            "`*` selects debugSymbols for {path}"
        );
    }

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
