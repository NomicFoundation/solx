//!
//! End-to-end DWARF debug info tests.
//!
//! These pin properties of the emitted `.debug_line` section as DWARF consumers read
//! it, after source locations have passed through the full pipeline: solc source
//! references, EVMLA translation, location selection, and LLVM optimization. They run
//! the compiler binary because contracts are compiled in subprocesses of the driver
//! executable, so there is no in-process path through the pipeline.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use object::Object;
use object::ObjectSection;
use tempfile::TempDir;
use test_case::test_case;

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

    let input = fixture(crate::common::standard_json!(
        "debug_info_cross_source_collision.json"
    ))?;
    let source_a = input.sources["A.sol"]
        .content
        .as_deref()
        .expect("Always exists");
    let source_b = input.sources["B.sol"]
        .content
        .as_deref()
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

    let output = compile_standard_json(&input)?;

    let row_counts = debug_line_row_counts(deployed_debug_info(&output, path, name)?.as_slice())?;
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

    let mut input = fixture(crate::common::standard_json!(
        "debug_info_generated_code.json"
    ))?;
    let source = input.sources["Probe.sol"]
        .content
        .clone()
        .expect("Always exists");
    let contract_declaration_line = line_of(&source, "contract Probe");
    let function_declaration_line = line_of(&source, "function set");
    // The Yul pipeline emits a sparser line table than EVMLA: of the probe's statements,
    // only the `if` guard keeps a row.
    let mut statement_lines = vec![line_of(&source, "if (newValue == 0)")];
    if !via_ir {
        statement_lines.push(line_of(&source, "value = newValue;"));
    }

    input.settings.via_ir = via_ir;
    let output = compile_standard_json(&input)?;

    let row_counts =
        debug_line_row_counts(deployed_debug_info(&output, "Probe.sol", "Probe")?.as_slice())?;

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
/// `outputSelection` must produce byte-identical bytecode. Both compilations are derived
/// from the same input and differ only in the output selection, so the invariance holds
/// by construction.
///
#[test_case(false ; "evmla")]
#[test_case(true ; "yul")]
fn bytecode_invariant_to_debug_info_selection(via_ir: bool) -> anyhow::Result<()> {
    crate::common::setup()?;

    let mut input = fixture(crate::common::standard_json!(
        "debug_info_generated_code.json"
    ))?;
    input.settings.via_ir = via_ir;

    let bytecode_selectors = BTreeSet::from([
        solx_standard_json::InputSelector::BytecodeObject,
        solx_standard_json::InputSelector::RuntimeBytecodeObject,
    ]);
    let mut debug_info_selectors = bytecode_selectors.clone();
    debug_info_selectors.insert(solx_standard_json::InputSelector::BytecodeDebugInfo);
    debug_info_selectors.insert(solx_standard_json::InputSelector::RuntimeBytecodeDebugInfo);

    let mut bytecode_with = |selectors: BTreeSet<solx_standard_json::InputSelector>| -> anyhow::Result<(String, String)> {
        input.settings.output_selection = solx_standard_json::InputSelection::new(selectors);
        let output = compile_standard_json(&input)?;
        let evm = output.contracts["Probe.sol"]["Probe"]
            .evm
            .as_ref()
            .expect("Always exists");
        Ok((
            evm.bytecode
                .as_ref()
                .and_then(|bytecode| bytecode.object.clone())
                .expect("Always exists"),
            evm.deployed_bytecode
                .as_ref()
                .and_then(|bytecode| bytecode.object.clone())
                .expect("Always exists"),
        ))
    };

    assert_eq!(
        bytecode_with(debug_info_selectors)?,
        bytecode_with(bytecode_selectors)?,
        "bytecode must not depend on the debugInfo output selection"
    );

    Ok(())
}

///
/// Reads a standard JSON fixture into the typed input.
///
fn fixture(path: &str) -> anyhow::Result<solx_standard_json::Input> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

///
/// Compiles a standard JSON input and returns the parsed output.
///
fn compile_standard_json(
    input: &solx_standard_json::Input,
) -> anyhow::Result<solx_standard_json::Output> {
    let input_directory = TempDir::with_prefix("solx_debug_info")?;
    let input_path = input_directory.path().join("input.json");
    std::fs::write(&input_path, serde_json::to_string(input)?)?;

    let args = &[
        "--standard-json",
        input_path.to_str().expect("Always valid"),
    ];
    let result = crate::cli::execute_solx(args)?.success();
    Ok(serde_json::from_slice(
        result.get_output().stdout.as_slice(),
    )?)
}

///
/// Extracts the deployed bytecode DWARF blob of the specified contract.
///
fn deployed_debug_info(
    output: &solx_standard_json::Output,
    path: &str,
    name: &str,
) -> anyhow::Result<Vec<u8>> {
    let debug_info = output.contracts[path][name]
        .evm
        .as_ref()
        .and_then(|evm| evm.deployed_bytecode.as_ref())
        .and_then(|bytecode| bytecode.debug_info.as_deref())
        .expect("Always exists");
    Ok(hex::decode(debug_info)?)
}

///
/// Returns the 1-based line number of the first source line containing `needle`.
///
fn line_of(source: &str, needle: &str) -> u64 {
    let index = source
        .lines()
        .position(|line| line.contains(needle))
        .unwrap_or_else(|| panic!("`{needle}` not found in the fixture source"));
    (index + 1) as u64
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
