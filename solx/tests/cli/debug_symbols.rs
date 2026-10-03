//!
//! CLI tests for the `debugSymbols` standard JSON output.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use predicates::prelude::*;
use solx_standard_json::output::source::debug_symbols::DebugSymbols;
use solx_standard_json::output::source::debug_symbols::contract::Contract;

const INPUT: &str = crate::common::standard_json!("debug_symbols.json");

#[test]
fn matches_expected() -> anyhow::Result<()> {
    crate::common::setup()?;

    let expected: BTreeMap<String, DebugSymbols> = serde_json::from_str(
        std::fs::read_to_string("tests/data/standard_json_output/debug_symbols.json")?.as_str(),
    )?;

    let output = compile(INPUT)?;
    let actual = output
        .sources
        .into_iter()
        .map(|(path, source)| (path, source.debug_symbols.expect("Always selected")))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn selectors_match_method_identifiers() -> anyhow::Result<()> {
    crate::common::setup()?;

    let output = compile(INPUT)?;
    let table = |file: &str, name: &str| -> &Contract {
        output.sources[file]
            .debug_symbols
            .as_ref()
            .expect("Always selected")
            .contracts
            .iter()
            .find(|contract| contract.name == name)
            .expect("Every linearised base has an entry")
    };

    let mut checked = 0;
    for (file, contracts) in output.contracts.iter() {
        for (name, contract) in contracts.iter() {
            let Some(method_identifiers) = contract
                .evm
                .as_ref()
                .and_then(|evm| evm.method_identifiers.as_ref())
            else {
                continue;
            };
            let identifiers = method_identifiers
                .values()
                .map(String::as_str)
                .collect::<BTreeSet<_>>();
            let reachable = table(file, name)
                .bases
                .iter()
                .flatten()
                .flat_map(|base| {
                    table(base.file.as_str(), base.name.as_str())
                        .functions
                        .iter()
                })
                .filter_map(|function| function.selector.as_deref())
                .collect::<BTreeSet<_>>();
            assert_eq!(identifiers, reachable, "selectors of `{file}:{name}`");
            checked += 1;
        }
    }
    assert_eq!(
        checked, 2,
        "`Counter` and `MathLib` have method identifiers"
    );

    Ok(())
}

#[test]
fn spans_cover_definitions_but_not_their_names() -> anyhow::Result<()> {
    crate::common::setup()?;

    let source = std::fs::read_to_string("tests/data/contracts/solidity/debug_symbols/Main.sol")?;
    let output = compile(INPUT)?;
    let symbols = output.sources["Main.sol"]
        .debug_symbols
        .as_ref()
        .expect("Always selected");
    let spans = symbols.spans.iter().collect::<BTreeSet<_>>();

    let counter = symbols
        .contracts
        .iter()
        .find(|contract| contract.name == "Counter")
        .expect("Always exists");
    assert!(spans.contains(&counter.range));
    for function in counter.functions.iter() {
        assert!(spans.contains(&function.range), "`{}`", function.name);
    }

    let name_offset = source.find("contract Counter").expect("Always exists") + "contract ".len();
    assert!(!spans.contains(&[name_offset, "Counter".len()]));

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

///
/// Compiles a standard JSON input and returns the parsed output.
///
fn compile(path: &str) -> anyhow::Result<solx_standard_json::Output> {
    let result = crate::cli::execute_solx(&["--standard-json", path])?.success();
    Ok(serde_json::from_slice(
        result.get_output().stdout.as_slice(),
    )?)
}
