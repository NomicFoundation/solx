//!
//! End-to-end DWARF debug info tests.
//!
//! These pin the emitted debug info, mostly the `.debug_line` section as DWARF consumers
//! read it, after source locations have passed through the full pipeline: the frontend's
//! locations, the Sol passes, LLVM optimization, and object emission. They run the
//! compiler binary because contracts are compiled in subprocesses of the driver
//! executable, so there is no in-process path through the pipeline.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use assert_cmd::assert::OutputAssertExt;
use object::Object;
use object::ObjectSection;
use tempfile::TempDir;
use test_case::test_case;

///
/// The fixture's two sources have the same length and put `contract` at the same offset while
/// their line numbers differ, so a location resolved in the wrong source lands in range, on the
/// other contract's lines.
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
/// The guard has a row on its own line. The dispatch has rows on the contract's declaration line
/// beyond `__entry`'s own, each dispatch case on its function's declaration line, a getter's on
/// its state variable's, and the ABI helpers on line 0.
///
#[test]
fn line_table() -> anyhow::Result<()> {
    crate::common::setup()?;

    let input = fixture(crate::common::standard_json!("debug_info_line_table.json"))?;
    let source = input.sources["LineTable.sol"]
        .content
        .clone()
        .expect("Always exists");
    let contract_declaration_line = line_of(&source, "contract LineTable");
    let getter_line = line_of(&source, "uint256 public value");
    let function_declaration_line = line_of(&source, "function set");
    let guard_line = line_of(&source, "if (newValue == 0)");

    let output = compile_standard_json(&input)?;

    let row_counts = debug_line_row_counts(
        deployed_debug_info(&output, "LineTable.sol", "LineTable")?.as_slice(),
    )?;

    assert!(
        row_counts.contains_key(&0),
        "the ABI helpers must produce line-0 rows: {row_counts:?}",
    );
    assert!(
        row_counts.contains_key(&guard_line),
        "the guard (line {guard_line}) is missing from the line table: {row_counts:?}",
    );
    assert!(
        row_counts
            .get(&contract_declaration_line)
            .copied()
            .unwrap_or_default()
            > 1,
        "the dispatch is missing from the contract declaration line {contract_declaration_line}: {row_counts:?}",
    );
    assert!(
        row_counts.contains_key(&function_declaration_line),
        "the dispatch case of `set` is missing from its declaration line {function_declaration_line}: {row_counts:?}",
    );
    assert!(
        row_counts.contains_key(&getter_line),
        "the dispatch case of the getter is missing from its state variable's line {getter_line}: {row_counts:?}",
    );

    Ok(())
}

///
/// Debug info requested for one code segment reaches that segment's LLVM IR only: its compile
/// unit and its DWARF version.
///
#[test_case(solx_standard_json::InputSelector::BytecodeDebugInfo ; "deploy")]
#[test_case(solx_standard_json::InputSelector::RuntimeBytecodeDebugInfo ; "runtime")]
fn debug_info_per_segment(requested: solx_standard_json::InputSelector) -> anyhow::Result<()> {
    crate::common::setup()?;

    let mut input = fixture(crate::common::standard_json!("debug_info_line_table.json"))?;
    input.settings.output_selection = solx_standard_json::InputSelection::new(BTreeSet::from([
        requested,
        solx_standard_json::InputSelector::BytecodeLLVMIR,
        solx_standard_json::InputSelector::RuntimeBytecodeLLVMIR,
    ]));
    let output = compile_standard_json(&input)?;

    let evm = output.contracts["LineTable.sol"]["LineTable"]
        .evm
        .as_ref()
        .expect("Always exists");
    let debug_info = |llvm_ir: Option<&str>| {
        let llvm_ir = llvm_ir.expect("Always exists");
        (
            llvm_ir.contains("!llvm.dbg.cu"),
            llvm_ir.contains(r#"!{i32 2, !"Dwarf Version", i32 5}"#),
        )
    };
    let deploy = requested == solx_standard_json::InputSelector::BytecodeDebugInfo;
    assert_eq!(
        debug_info(
            evm.bytecode
                .as_ref()
                .and_then(|bytecode| bytecode.llvm_ir.as_deref())
        ),
        (deploy, deploy),
        "debug info of the deploy code",
    );
    let runtime = requested == solx_standard_json::InputSelector::RuntimeBytecodeDebugInfo;
    assert_eq!(
        debug_info(
            evm.deployed_bytecode
                .as_ref()
                .and_then(|bytecode| bytecode.llvm_ir.as_deref())
        ),
        (runtime, runtime),
        "debug info of the runtime code",
    );

    Ok(())
}

///
/// The statements of an `assembly {}` body keep rows on their own lines, a Yul function's body
/// included: its `yul.func` carries a subprogram of its own, without which the function it is
/// lowered to drops its body's locations. The `revert(0, 0)` row is not asserted: the backend
/// merges identical revert sequences, and LLVM puts a location merged from different lines on
/// line 0.
///
#[test]
fn inline_assembly_lines() -> anyhow::Result<()> {
    crate::common::setup()?;

    let input = fixture(crate::common::standard_json!(
        "debug_info_inline_assembly.json"
    ))?;
    let source = input.sources["AsmProbe.sol"]
        .content
        .clone()
        .expect("Always exists");

    let output = compile_standard_json(&input)?;

    let row_counts = debug_line_row_counts(
        deployed_debug_info(&output, "AsmProbe.sol", "AsmProbe")?.as_slice(),
    )?;

    for needle in [
        "if gt(v, 100)",
        "if gt(newValue, 1000)",
        "sstore(1, clamp(newValue))",
        "value = newValue;",
    ] {
        let line = line_of(&source, needle);
        assert!(
            row_counts.contains_key(&line),
            "`{needle}` (line {line}) is missing from the line table: {row_counts:?}",
        );
    }

    Ok(())
}

///
/// A modifier's body is inlined into the function it modifies, and its statements keep rows on
/// their own lines in that function's line table.
///
#[test]
fn modifier_lines() -> anyhow::Result<()> {
    crate::common::setup()?;

    let input = fixture(crate::common::standard_json!("debug_info_modifier.json"))?;
    let source = input.sources["Modified.sol"]
        .content
        .clone()
        .expect("Always exists");

    let output = compile_standard_json(&input)?;

    let row_counts = debug_line_row_counts(
        deployed_debug_info(&output, "Modified.sol", "Modified")?.as_slice(),
    )?;

    for needle in ["require(x > 0)", "value += 1"] {
        let line = line_of(&source, needle);
        assert!(
            row_counts.contains_key(&line),
            "`{needle}` (line {line}) is missing from the line table: {row_counts:?}",
        );
    }

    Ok(())
}

///
/// Code from another source than its function's keeps its rows in its own source: an inherited
/// modifier inlined into a function, a base's state-variable initializer in the derived
/// constructor, and a base's constant lowered where it is used.
///
#[test_case(solx_utils::CodeSegment::Runtime, "count += 1;" ; "inherited_modifier")]
#[test_case(solx_utils::CodeSegment::Deploy, "start = block.number;" ; "base_initializer")]
#[test_case(solx_utils::CodeSegment::Runtime, "SEED = sha256" ; "base_constant")]
fn cross_file_lines(code_segment: solx_utils::CodeSegment, needle: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let input = fixture(crate::common::standard_json!("debug_info_cross_file.json"))?;
    let source = input.sources["Base.sol"]
        .content
        .clone()
        .expect("Always exists");
    let line = line_of(&source, needle);

    let output = compile_standard_json(&input)?;

    let evm = output.contracts["Derived.sol"]["Derived"]
        .evm
        .as_ref()
        .expect("Always exists");
    let bytecode = match code_segment {
        solx_utils::CodeSegment::Deploy => evm.bytecode.as_ref(),
        solx_utils::CodeSegment::Runtime => evm.deployed_bytecode.as_ref(),
    };
    let debug_info = hex::decode(
        bytecode
            .and_then(|bytecode| bytecode.debug_info.as_deref())
            .expect("Always exists"),
    )?;
    let rows = debug_line_rows(debug_info.as_slice())?;

    assert!(
        rows.contains(&("Base.sol".to_owned(), line)),
        "`{needle}` (Base.sol line {line}) is missing from the {code_segment} line table: {rows:?}",
    );

    Ok(())
}

///
/// Debug info must never influence codegen: compiling with and without `debugInfo` in
/// `outputSelection` must produce byte-identical bytecode.
///
#[test]
fn bytecode_invariant_to_debug_info_selection() -> anyhow::Result<()> {
    crate::common::setup()?;

    let mut input = fixture(crate::common::standard_json!("debug_info_line_table.json"))?;

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
        let evm = output.contracts["LineTable.sol"]["LineTable"]
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

    let with_debug_info = bytecode_with(debug_info_selectors)?;
    let without_debug_info = bytecode_with(bytecode_selectors)?;
    assert_eq!(
        with_debug_info, without_debug_info,
        "bytecode must not depend on the debugInfo output selection"
    );

    Ok(())
}

///
/// Debug info must not depend on the compiler's working directory. Standard JSON source
/// names are the complete source identity, so identical inputs compiled from different
/// directories must produce byte-identical DWARF.
///
#[test]
fn debug_info_invariant_to_working_directory() -> anyhow::Result<()> {
    crate::common::setup()?;

    let input = fixture(crate::common::standard_json!("debug_info_line_table.json"))?;

    let first_directory = TempDir::with_prefix("solx_debug_info_first")?;
    let second_directory = TempDir::with_prefix("solx_debug_info_second")?;
    let first_output = compile_standard_json_in(&input, first_directory.path())?;
    let second_output = compile_standard_json_in(&input, second_directory.path())?;

    assert_eq!(
        deployed_debug_info(&first_output, "LineTable.sol", "LineTable")?,
        deployed_debug_info(&second_output, "LineTable.sol", "LineTable")?,
        "debug info must not depend on the compiler working directory",
    );

    Ok(())
}

///
/// A line ends at `\n`, `\r\n` or a lone `\r`. Whichever the source uses, each row lands on the
/// same line and column, so the debug info is byte-identical to that of the `\n` source.
///
#[test_case("\r\n" ; "crlf")]
#[test_case("\r" ; "cr")]
fn debug_info_invariant_to_line_terminators(terminator: &str) -> anyhow::Result<()> {
    crate::common::setup()?;

    let input = fixture(crate::common::standard_json!("debug_info_line_table.json"))?;
    let mut terminated = fixture(crate::common::standard_json!("debug_info_line_table.json"))?;
    let source = terminated
        .sources
        .get_mut("LineTable.sol")
        .and_then(|source| source.content.as_mut())
        .expect("Always exists");
    *source = source.replace('\n', terminator);

    let output = compile_standard_json(&input)?;
    let terminated_output = compile_standard_json(&terminated)?;

    assert_eq!(
        deployed_debug_info(&terminated_output, "LineTable.sol", "LineTable")?,
        deployed_debug_info(&output, "LineTable.sol", "LineTable")?,
        "debug info must not depend on the source's line terminators",
    );

    Ok(())
}

///
/// Reads a standard JSON fixture into the typed input.
///
fn fixture(path: &str) -> anyhow::Result<solx_standard_json::Input> {
    solx_standard_json::Input::try_from(Some(std::path::Path::new(path)))
}

///
/// Compiles a standard JSON input and returns the parsed output.
///
fn compile_standard_json(
    input: &solx_standard_json::Input,
) -> anyhow::Result<solx_standard_json::Output> {
    let current_directory = std::env::current_dir()?;
    compile_standard_json_in(input, current_directory.as_path())
}

///
/// Compiles `input` with the compiler running in `working_directory`.
///
fn compile_standard_json_in(
    input: &solx_standard_json::Input,
    working_directory: &Path,
) -> anyhow::Result<solx_standard_json::Output> {
    let input_directory = TempDir::with_prefix("solx_debug_info")?;
    let input_path = input_directory.path().join("input.json");
    std::fs::write(&input_path, serde_json::to_string(input)?)?;

    let args = &[
        "--standard-json",
        input_path.to_str().expect("Always valid"),
    ];
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!(env!("CARGO_PKG_NAME")));
    let result = command
        .current_dir(working_directory)
        .args(args)
        .assert()
        .success();
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
/// Counts the DWARF `.debug_line` program rows per line number, line 0 for rows without a source
/// association.
///
fn debug_line_row_counts(elf: &[u8]) -> anyhow::Result<BTreeMap<u64, usize>> {
    let mut row_counts = BTreeMap::new();
    for (_, line) in debug_line_rows(elf)? {
        *row_counts.entry(line).or_insert(0usize) += 1;
    }
    Ok(row_counts)
}

///
/// The source file name and line of each DWARF `.debug_line` program row, line 0 for rows without
/// a source association.
///
fn debug_line_rows(elf: &[u8]) -> anyhow::Result<Vec<(String, u64)>> {
    let dwarf = dwarf(elf)?;

    let mut rows = Vec::new();
    let mut units = dwarf.units();
    while let Some(unit_header) = units.next()? {
        let unit = dwarf.unit(unit_header)?;
        let Some(program) = unit.line_program.clone() else {
            continue;
        };
        let mut program_rows = program.rows();
        while let Some((header, row)) = program_rows.next_row()? {
            if row.end_sequence() {
                continue;
            }
            let file = row.file(header).expect("Always exists");
            let name = dwarf.attr_string(&unit, file.path_name())?;
            let line = row.line().map_or(0, std::num::NonZeroU64::get);
            rows.push((name.to_string_lossy().into_owned(), line));
        }
    }
    Ok(rows)
}

///
/// Loads the DWARF sections of a debug info ELF.
///
fn dwarf(elf: &[u8]) -> anyhow::Result<gimli::Dwarf<gimli::EndianSlice<'_, gimli::RunTimeEndian>>> {
    let object_file = object::File::parse(elf)?;
    let endian = if object_file.is_little_endian() {
        gimli::RunTimeEndian::Little
    } else {
        gimli::RunTimeEndian::Big
    };
    Ok(gimli::Dwarf::load(|section| -> gimli::Result<_> {
        let data = object_file
            .section_by_name(section.name())
            .and_then(|section| section.data().ok())
            .unwrap_or_default();
        Ok(gimli::EndianSlice::new(data, endian))
    })?)
}
