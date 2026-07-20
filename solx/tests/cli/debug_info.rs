//!
//! CLI tests for the eponymous option.
//!

use std::collections::BTreeMap;

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

    let fixture_path = crate::common::standard_json!("debug_info_cross_source_collision.json");
    let fixture: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(fixture_path)?)?;
    let source_a = fixture["sources"]["A.sol"]["content"]
        .as_str()
        .expect("Always exists");
    let source_b = fixture["sources"]["B.sol"]["content"]
        .as_str()
        .expect("Always exists");
    assert_eq!(
        source_a.len(),
        source_b.len(),
        "the sources must stay byte-length-identical: without the offset collision this test passes vacuously",
    );
    assert_eq!(
        source_a.find("contract"),
        source_b.find("contract"),
        "the sources must keep their AST byte offsets aligned: without the offset collision this test passes vacuously",
    );

    let args = &["--standard-json", fixture_path];

    let result = crate::cli::execute_solx(args)?.success();
    let output: serde_json::Value = serde_json::from_slice(result.get_output().stdout.as_slice())?;

    let debug_info = output["contracts"][path][name]["evm"]["deployedBytecode"]["debugInfo"]
        .as_str()
        .expect("Always exists");
    let row_counts = debug_line_row_counts(hex::decode(debug_info)?.as_slice())?;
    let lines: Vec<u64> = row_counts
        .keys()
        .copied()
        .filter(|&line| line != 0)
        .collect();

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
/// The fixture's contract exposes a public function, so the bytecode contains selector
/// dispatch and ABI-decoding code with no Solidity statement behind it. Such code must
/// carry DWARF line 0 — the convention for "no source association" — instead of
/// inheriting the contract or function declaration line. The only permitted
/// declaration-line row is the artificial subprogram anchor.
///
#[test_case(false ; "evmla")]
#[test_case(true ; "yul")]
fn generated_code_line_zero(via_ir: bool) -> anyhow::Result<()> {
    crate::common::setup()?;

    let fixture_path = crate::common::standard_json!("debug_info_generated_code.json");
    let mut fixture: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(fixture_path)?)?;
    let source = fixture["sources"]["Probe.sol"]["content"]
        .as_str()
        .expect("Always exists")
        .to_owned();
    let line_of = |needle: &str| -> u64 {
        let index = source
            .lines()
            .position(|line| line.contains(needle))
            .unwrap_or_else(|| panic!("`{needle}` not found in the fixture source"));
        (index + 1) as u64
    };
    let contract_declaration_line = line_of("contract Probe");
    let function_declaration_line = line_of("function set");
    // The Yul pipeline emits a sparser line table than EVMLA: of the probe's statements,
    // only the `if` guard keeps a row.
    let mut statement_lines = vec![line_of("if (newValue == 0)")];
    if !via_ir {
        statement_lines.push(line_of("value = newValue;"));
    }

    let input_directory = TempDir::with_prefix("solx_debug_line_zero")?;
    let input_path = input_directory.path().join("input.json");
    fixture["settings"]["viaIR"] = serde_json::Value::Bool(via_ir);
    std::fs::write(&input_path, serde_json::to_string(&fixture)?)?;

    let args = &[
        "--standard-json",
        input_path.to_str().expect("Always valid"),
    ];

    let result = crate::cli::execute_solx(args)?.success();
    let output: serde_json::Value = serde_json::from_slice(result.get_output().stdout.as_slice())?;

    let debug_info =
        output["contracts"]["Probe.sol"]["Probe"]["evm"]["deployedBytecode"]["debugInfo"]
            .as_str()
            .expect("Always exists");
    let row_counts = debug_line_row_counts(hex::decode(debug_info)?.as_slice())?;

    assert!(
        row_counts.contains_key(&0),
        "generated code must produce line-0 rows: {row_counts:?}",
    );
    assert!(
        !row_counts.contains_key(&function_declaration_line),
        "generated code is attributed to the function declaration line {function_declaration_line}: {row_counts:?}",
    );
    assert!(
        row_counts
            .get(&contract_declaration_line)
            .copied()
            .unwrap_or_default()
            <= 1,
        "generated code is attributed to the contract declaration line {contract_declaration_line}: {row_counts:?}",
    );
    for statement_line in statement_lines {
        assert!(
            row_counts.contains_key(&statement_line),
            "statement line {statement_line} is missing from the line table: {row_counts:?}",
        );
    }

    Ok(())
}

///
/// Debug info must never influence codegen: compiling with and without `debugInfo` in
/// `outputSelection` must produce byte-identical bytecode.
///
#[test_case(false ; "evmla")]
#[test_case(true ; "yul")]
fn bytecode_invariant_to_debug_info_selection(via_ir: bool) -> anyhow::Result<()> {
    crate::common::setup()?;

    let fixture_path = crate::common::standard_json!("debug_info_generated_code.json");
    let fixture: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(fixture_path)?)?;

    let input_directory = TempDir::with_prefix("solx_debug_info_invariance")?;
    let compile = |with_debug_info: bool| -> anyhow::Result<(String, String)> {
        let mut selection = vec!["evm.bytecode.object", "evm.deployedBytecode.object"];
        if with_debug_info {
            selection.push("evm.bytecode.debugInfo");
            selection.push("evm.deployedBytecode.debugInfo");
        }
        let mut input = fixture.clone();
        input["settings"]["viaIR"] = serde_json::Value::Bool(via_ir);
        input["settings"]["outputSelection"]["*"]["*"] = serde_json::json!(selection);
        let input_path = input_directory
            .path()
            .join(format!("input_{with_debug_info}.json"));
        std::fs::write(&input_path, serde_json::to_string(&input)?)?;

        let args = &[
            "--standard-json",
            input_path.to_str().expect("Always valid"),
        ];
        let result = crate::cli::execute_solx(args)?.success();
        let output: serde_json::Value =
            serde_json::from_slice(result.get_output().stdout.as_slice())?;
        let evm = &output["contracts"]["Probe.sol"]["Probe"]["evm"];
        Ok((
            evm["bytecode"]["object"]
                .as_str()
                .expect("Always exists")
                .to_owned(),
            evm["deployedBytecode"]["object"]
                .as_str()
                .expect("Always exists")
                .to_owned(),
        ))
    };

    let with_debug_info = compile(true)?;
    let without_debug_info = compile(false)?;
    assert_eq!(
        with_debug_info, without_debug_info,
        "bytecode must not depend on the debugInfo output selection"
    );

    Ok(())
}

///
/// Counts the DWARF `.debug_line` program rows per line number. Rows without a source
/// association (DWARF line 0) are keyed as `0`.
///
fn debug_line_row_counts(elf: &[u8]) -> anyhow::Result<BTreeMap<u64, usize>> {
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

    let mut row_counts = BTreeMap::new();
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
            let line = row.line().map_or(0, std::num::NonZeroU64::get);
            *row_counts.entry(line).or_insert(0usize) += 1;
        }
    }
    Ok(row_counts)
}
