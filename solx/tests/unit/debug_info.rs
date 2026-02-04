//!
//! Unit tests for debug info generation.
//!
//! These tests verify that DWARF debug information is correctly generated
//! and that all function entries have valid bytecode offsets.
//!

use std::collections::HashMap;

use gimli::EndianSlice;
use gimli::RunTimeEndian;
use object::Object;
use object::ObjectSection;
use test_case::test_case;

#[test_case(true ; "yul")]
#[test_case(false ; "evmla")]
fn debug_info_simple_contract(via_ir: bool) {
    let sources =
        crate::common::read_sources(&["tests/data/contracts/solidity/SimpleContract.sol"]);
    let output = crate::common::build_solidity_standard_json_debug_info(sources, via_ir)
        .expect("Build failed");

    let contracts = output
        .contracts
        .get("tests/data/contracts/solidity/SimpleContract.sol")
        .expect("Contract file not found");

    let contract = contracts.get("SimpleContract").expect("Contract not found");

    let evm = contract.evm.as_ref().expect("EVM output not found");

    // Check deploy bytecode debug info
    let bytecode = evm.bytecode.as_ref().expect("Bytecode not found");
    let debug_info_hex = bytecode
        .debug_info
        .as_ref()
        .expect("Debug info not found in bytecode");
    let bytecode_hex = bytecode.object.as_ref().expect("Bytecode object not found");
    let bytecode_length = get_bytecode_length(bytecode_hex);

    let functions = parse_debug_info_functions(debug_info_hex).expect("Failed to parse debug info");

    assert!(!functions.is_empty(), "No functions found in debug info");

    // Verify all function addresses are within bytecode bounds
    for (function_name, address) in functions.iter() {
        assert!(
            (*address as usize) < bytecode_length,
            "Function '{}' has address {} which is outside bytecode length {}",
            function_name,
            address,
            bytecode_length
        );
    }

    // Check runtime bytecode debug info
    let deployed_bytecode = evm
        .deployed_bytecode
        .as_ref()
        .expect("Deployed bytecode not found");
    let runtime_debug_info_hex = deployed_bytecode
        .debug_info
        .as_ref()
        .expect("Debug info not found in deployed bytecode");
    let runtime_bytecode_hex = deployed_bytecode
        .object
        .as_ref()
        .expect("Deployed bytecode object not found");
    let runtime_bytecode_length = get_bytecode_length(runtime_bytecode_hex);

    let runtime_functions = parse_debug_info_functions(runtime_debug_info_hex)
        .expect("Failed to parse runtime debug info");

    assert!(
        !runtime_functions.is_empty(),
        "No functions found in runtime debug info"
    );

    // Verify all runtime function addresses are within runtime bytecode bounds
    for (function_name, address) in runtime_functions.iter() {
        assert!(
            (*address as usize) < runtime_bytecode_length,
            "Runtime function '{}' has address {} which is outside runtime bytecode length {}",
            function_name,
            address,
            runtime_bytecode_length
        );
    }
}

#[test_case(true ; "yul")]
#[test_case(false ; "evmla")]
fn debug_info_complex_contract(via_ir: bool) {
    let sources = crate::common::read_sources(&["tests/data/contracts/solidity/Test.sol"]);
    let output = crate::common::build_solidity_standard_json_debug_info(sources, via_ir)
        .expect("Build failed");

    let contracts = output
        .contracts
        .get("tests/data/contracts/solidity/Test.sol")
        .expect("Contract file not found");

    let contract = contracts.get("Test").expect("Contract not found");

    let evm = contract.evm.as_ref().expect("EVM output not found");

    // Check runtime bytecode debug info (more interesting for complex contracts)
    let deployed_bytecode = evm
        .deployed_bytecode
        .as_ref()
        .expect("Deployed bytecode not found");
    let debug_info_hex = deployed_bytecode
        .debug_info
        .as_ref()
        .expect("Debug info not found in deployed bytecode");
    let bytecode_hex = deployed_bytecode
        .object
        .as_ref()
        .expect("Deployed bytecode object not found");
    let bytecode_length = get_bytecode_length(bytecode_hex);

    let functions = parse_debug_info_functions(debug_info_hex).expect("Failed to parse debug info");

    // The Test contract has 'entry' and 'test' functions
    assert!(
        functions.len() >= 2,
        "Expected at least 2 functions, found {}",
        functions.len()
    );

    // Verify all function addresses are within bytecode bounds
    for (function_name, address) in functions.iter() {
        assert!(
            (*address as usize) < bytecode_length,
            "Function '{}' has address {} which is outside bytecode length {}",
            function_name,
            address,
            bytecode_length
        );
    }
}

#[test_case(true ; "yul")]
#[test_case(false ; "evmla")]
fn debug_info_multiple_contracts(via_ir: bool) {
    let sources = crate::common::read_sources(&[
        "tests/data/contracts/solidity/caller/Main.sol",
        "tests/data/contracts/solidity/caller/Callable.sol",
    ]);
    let output = crate::common::build_solidity_standard_json_debug_info(sources, via_ir)
        .expect("Build failed");

    // Check that both contracts have debug info
    for (file_path, contracts) in output.contracts.iter() {
        for (contract_name, contract) in contracts.iter() {
            let Some(evm) = contract.evm.as_ref() else {
                panic!("EVM output not found for {file_path}:{contract_name}");
            };
            let Some(deployed_bytecode) = evm.deployed_bytecode.as_ref() else {
                continue;
            };
            let Some(debug_info_hex) = deployed_bytecode.debug_info.as_ref() else {
                continue;
            };
            let Some(bytecode_hex) = deployed_bytecode.object.as_ref() else {
                continue;
            };
            if bytecode_hex.is_empty() {
                continue; // Skip contracts with no bytecode (interfaces, etc.)
            }

            let bytecode_length = get_bytecode_length(bytecode_hex);
            let functions = parse_debug_info_functions(debug_info_hex).unwrap_or_else(|error| {
                panic!("Failed to parse debug info for {file_path}:{contract_name}: {error}")
            });

            // Verify all function addresses are within bytecode bounds
            for (function_name, address) in functions.iter() {
                assert!(
                    (*address as usize) < bytecode_length,
                    "Contract {file_path}:{contract_name} function '{}' has address {} which is outside bytecode length {}",
                    function_name,
                    address,
                    bytecode_length
                );
            }
        }
    }
}

/// Parses the hex-encoded debug info and extracts function names with their low_pc addresses.
/// Returns a map of function name -> low_pc address.
fn parse_debug_info_functions(debug_info_hex: &str) -> anyhow::Result<HashMap<String, u64>> {
    let debug_info_bytes = hex::decode(debug_info_hex)?;
    let object_file = object::File::parse(&*debug_info_bytes)?;

    // Detect endianness from the ELF file
    let endian = if object_file.is_little_endian() {
        RunTimeEndian::Little
    } else {
        RunTimeEndian::Big
    };

    // Helper to get section data as EndianSlice
    let get_section = |name: &str| -> gimli::Result<EndianSlice<'_, RunTimeEndian>> {
        let data = object_file
            .section_by_name(name)
            .and_then(|section| section.data().ok())
            .unwrap_or(&[]);
        Ok(EndianSlice::new(data, endian))
    };

    // Load all DWARF sections
    let dwarf = gimli::Dwarf::load(|section_id| get_section(section_id.name()))?;

    let mut functions = HashMap::new();
    let mut units = dwarf.units();

    while let Some(unit_header) = units.next()? {
        let unit = dwarf.unit(unit_header)?;
        let mut entries = unit.entries();

        while let Some((_, entry)) = entries.next_dfs()? {
            if entry.tag() == gimli::DW_TAG_subprogram {
                let mut name: Option<String> = None;
                let mut low_pc: Option<u64> = None;

                let mut attrs = entry.attrs();
                while let Some(attr) = attrs.next()? {
                    match attr.name() {
                        gimli::DW_AT_name => {
                            if let Ok(value) = dwarf.attr_string(&unit, attr.value()) {
                                name = Some(value.to_string_lossy().into_owned());
                            }
                        }
                        gimli::DW_AT_low_pc => {
                            if let Ok(Some(address)) = dwarf.attr_address(&unit, attr.value()) {
                                low_pc = Some(address);
                            }
                        }
                        gimli::DW_AT_linkage_name => {
                            // Fallback to linkage name if name not set
                            if name.is_none()
                                && let Ok(value) = dwarf.attr_string(&unit, attr.value())
                            {
                                name = Some(value.to_string_lossy().into_owned());
                            }
                        }
                        _ => {}
                    }
                }

                if let (Some(function_name), Some(address)) = (name, low_pc) {
                    functions.insert(function_name, address);
                }
            }
        }
    }

    Ok(functions)
}

/// Gets the bytecode length from a hex-encoded bytecode string.
fn get_bytecode_length(bytecode_hex: &str) -> usize {
    bytecode_hex.len() / 2
}
