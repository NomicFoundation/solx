//!
//! DWARF debug information validator for compiled contracts.
//!

use gimli::EndianSlice;
use gimli::RunTimeEndian;
use object::Object;
use object::ObjectSection;

use solx_standard_json::Output;

///
/// Validates DWARF debug information embedded in compiled contract bytecodes.
///
pub struct DwarfValidator;

impl DwarfValidator {
    ///
    /// Validates debug information for all contracts in the standard JSON output.
    ///
    /// Iterates every contract's deploy and runtime bytecodes, parses the
    /// hex-encoded DWARF ELF, and checks that all function `low_pc` addresses
    /// fall within the bytecode bounds.
    ///
    /// Contracts with empty bytecode (interfaces, abstract contracts) are skipped.
    ///
    /// # Errors
    ///
    /// Returns an error if any contract's debug information is malformed or
    /// contains out-of-bounds function addresses.
    ///
    pub fn validate_output(output: &Output) -> anyhow::Result<()> {
        for (file_path, contracts) in output.contracts.iter() {
            for (contract_name, contract) in contracts.iter() {
                let label = format!("{file_path}:{contract_name}");

                let Some(evm) = contract.evm.as_ref() else {
                    continue;
                };

                for (bytecode_container, code_segment) in [
                    (&evm.bytecode, solx_utils::CodeSegment::Deploy),
                    (&evm.deployed_bytecode, solx_utils::CodeSegment::Runtime),
                ] {
                    if let Some(bytecode_container) = bytecode_container.as_ref()
                        && let Some(bytecode) = bytecode_container.object.as_ref()
                        && let Some(debug_information) = bytecode_container.debug_info.as_ref()
                        && !bytecode.is_empty()
                    {
                        Self::validate_bytecode_debug_information(
                            debug_information,
                            bytecode,
                            &format!("{label}.{code_segment}"),
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    ///
    /// Validates a single bytecode's DWARF debug information.
    ///
    /// Parses the hex-encoded debug information and verifies that all function
    /// `low_pc` addresses are within the bytecode bounds. Empty DWARF (no
    /// subprogram entries) is valid — the optimizer may inline all functions.
    ///
    /// # Errors
    ///
    /// Returns an error if the debug information cannot be parsed or
    /// a function address exceeds the bytecode length.
    ///
    fn validate_bytecode_debug_information(
        debug_information_hex: &str,
        bytecode_hex: &str,
        contract_label: &str,
    ) -> anyhow::Result<()> {
        let functions = Self::parse_debug_information_functions(debug_information_hex)?;

        let bytecode_length = bytecode_hex.len() / 2;
        for (function_name, address) in functions.iter() {
            anyhow::ensure!(
                (*address as usize) < bytecode_length,
                "{contract_label} function '{function_name}' has address {address} \
                 which is outside bytecode length {bytecode_length}",
            );
        }

        Ok(())
    }

    ///
    /// Parses hex-encoded debug information and extracts function names with their
    /// `low_pc` addresses.
    ///
    /// Returns a list of `(name, low_pc)` pairs for subprograms and inlined subroutines.
    ///
    /// # Errors
    ///
    /// Returns an error if the hex string cannot be decoded, the ELF cannot
    /// be parsed, or the DWARF sections are malformed.
    ///
    fn parse_debug_information_functions(
        debug_information_hex: &str,
    ) -> anyhow::Result<Vec<(String, u64)>> {
        let debug_information_bytes = hex::decode(debug_information_hex)?;
        let object_file = object::File::parse(&*debug_information_bytes)?;

        let endian = if object_file.is_little_endian() {
            RunTimeEndian::Little
        } else {
            RunTimeEndian::Big
        };

        let load_section = |name: &str| -> gimli::Result<EndianSlice<'_, RunTimeEndian>> {
            let data = object_file
                .section_by_name(name)
                .and_then(|section| section.data().ok())
                .unwrap_or(&[]);
            Ok(EndianSlice::new(data, endian))
        };

        let dwarf =
            gimli::Dwarf::load(|section_identifier| load_section(section_identifier.name()))?;

        let mut functions = Vec::new();
        let mut units = dwarf.units();

        while let Some(unit_header) = units.next()? {
            let unit = dwarf.unit(unit_header)?;
            let mut entries = unit.entries();

            while let Some(entry) = entries.next_dfs()? {
                if entry.tag() != gimli::DW_TAG_subprogram
                    && entry.tag() != gimli::DW_TAG_inlined_subroutine
                {
                    continue;
                }

                let mut name: Option<String> = None;
                let mut low_pc: Option<u64> = None;

                for attribute in entry.attrs().iter() {
                    match attribute.name() {
                        gimli::DW_AT_name => {
                            if let Ok(value) = dwarf.attr_string(&unit, attribute.value()) {
                                name = Some(value.to_string_lossy().into_owned());
                            }
                        }
                        gimli::DW_AT_low_pc => {
                            if let Ok(Some(address)) = dwarf.attr_address(&unit, attribute.value())
                            {
                                low_pc = Some(address);
                            }
                        }
                        gimli::DW_AT_linkage_name => {
                            if name.is_none()
                                && let Ok(value) = dwarf.attr_string(&unit, attribute.value())
                            {
                                name = Some(value.to_string_lossy().into_owned());
                            }
                        }
                        _ => {}
                    }
                }

                if let Some(address) = low_pc {
                    let function_name = name.unwrap_or_else(|| "<inlined>".to_owned());
                    functions.push((function_name, address));
                }
            }
        }

        Ok(functions)
    }
}
