//!
//! CLI tests for the eponymous option.
//!

use std::collections::BTreeSet;

use object::Object;
use object::ObjectSection;
use predicates::prelude::*;
use tempfile::TempDir;
use test_case::test_case;

#[test_case(true ; "yul")]
#[test_case(false ; "evmla")]
fn default(via_ir: bool) -> anyhow::Result<()> {
    crate::common::setup()?;

    let mut args = vec![crate::common::TEST_SOLIDITY_CONTRACT, "--debug-info"];
    if via_ir {
        args.push("--via-ir");
    }

    let result = crate::cli::execute_solx(&args)?;

    result
        .success()
        .stdout(predicate::str::contains("Debug info").count(1));

    Ok(())
}

#[test]
fn standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_SOLIDITY_STANDARD_JSON,
        "--debug-info",
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

    let output_directory = TempDir::with_prefix("solx_debug_output")?;

    let args = &[
        crate::common::TEST_SOLIDITY_CONTRACT,
        "--debug-info",
        "--output-dir",
        output_directory.path().to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stderr(predicate::str::contains("Compiler run successful"));

    Ok(())
}

///
/// The fixture's two sources have byte-identical lengths, so every AST node byte offset in
/// `B.sol` collides with one in `A.sol`, while their line numbers differ. Keying AST nodes
/// by byte offset alone lets one source's locations overwrite the other's, attributing
/// `Alpha`'s DWARF line rows to `Betaa`'s declaration lines.
///
#[test_case("A.sol", "Alpha", 4, 6 ; "first_source")]
#[test_case("B.sol", "Betaa", 8, 10 ; "second_source")]
fn cross_source_line_attribution(
    path: &str,
    name: &str,
    first_line: u64,
    last_line: u64,
) -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("debug_info_cross_source_collision.json"),
    ];

    let result = crate::cli::execute_solx(args)?.success();
    let output: serde_json::Value = serde_json::from_slice(result.get_output().stdout.as_slice())?;

    let debug_info = output["contracts"][path][name]["evm"]["deployedBytecode"]["debugInfo"]
        .as_str()
        .expect("Always exists");
    let lines = debug_line_numbers(hex::decode(debug_info)?.as_slice())?;

    assert!(!lines.is_empty(), "{name} has an empty DWARF line table");
    for line in lines {
        assert!(
            (first_line..=last_line).contains(&line),
            "{name} line {line} is outside of its source range {first_line}..={last_line}",
        );
    }

    Ok(())
}

///
/// Collects the distinct non-zero line numbers from the DWARF `.debug_line` program.
///
fn debug_line_numbers(elf: &[u8]) -> anyhow::Result<BTreeSet<u64>> {
    let object_file = object::File::parse(elf)?;
    let endian = if object_file.is_little_endian() {
        gimli::RunTimeEndian::Little
    } else {
        gimli::RunTimeEndian::Big
    };
    let dwarf = gimli::Dwarf::load(|section| -> gimli::Result<_> {
        let data = object_file
            .section_by_name(section.name())
            .and_then(|section| section.data().ok())
            .unwrap_or_default();
        Ok(gimli::EndianSlice::new(data, endian))
    })?;

    let mut lines = BTreeSet::new();
    let mut units = dwarf.units();
    while let Some(unit_header) = units.next()? {
        let unit = dwarf.unit(unit_header)?;
        let Some(program) = unit.line_program.clone() else {
            continue;
        };
        let mut rows = program.rows();
        while let Some((_, row)) = rows.next_row()? {
            if row.end_sequence() {
                continue;
            }
            if let Some(line) = row.line() {
                lines.insert(line.get());
            }
        }
    }
    Ok(lines)
}
