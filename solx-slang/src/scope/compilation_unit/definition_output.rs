//!
//! One definition's share of the standard-JSON output: its contract and the benchmarks its
//! lowering recorded, with the member that locates it.
//!

use std::collections::BTreeMap;

use slang_solidity_v2::ast::NodeLocation;
use slang_solidity_v2::ast::SourceUnitMember;

use solx_standard_json::OutputError;
use solx_standard_json::output::contract::Contract;
use solx_standard_json::output::error::source_location::SourceLocation;

use crate::debug_locations::resolver::Resolver;
use crate::error::Error;

/// One definition's share of the standard-JSON output.
pub struct DefinitionOutput {
    /// The member declaring the definition, which locates it.
    pub member: SourceUnitMember,
    /// The definition's name, which keys its contract.
    pub name: String,
    /// The definition's contract, or the output Slang cannot compute.
    pub contract: Result<Contract, Error>,
    /// The benchmarks the lowering recorded, when they are requested.
    pub benchmarks: Vec<(String, u64)>,
}

impl DefinitionOutput {
    /// The share of the definition `member` declares under `name`.
    pub fn new(
        member: SourceUnitMember,
        name: String,
        contract: Result<Contract, Error>,
        benchmarks: Vec<(String, u64)>,
    ) -> Self {
        Self {
            member,
            name,
            contract,
            benchmarks,
        }
    }

    /// Writes the contract to `standard_json` under the definition's file and name, or the output
    /// Slang cannot compute as an error at the definition quoting `input_sources`, and the
    /// benchmarks.
    pub fn write_to_standard_json(
        self,
        standard_json: &mut solx_standard_json::Output,
        input_sources: &BTreeMap<String, solx_standard_json::InputSource>,
    ) {
        match self.contract {
            Ok(contract) => {
                let file_id = self
                    .member
                    .calculate_file_id()
                    .expect("slang locates every definition");
                standard_json
                    .contracts
                    .entry(file_id.to_string())
                    .or_default()
                    .insert(self.name, contract);
            }
            Err(error) => {
                let (file_id, text_range) = Resolver::source_range(&self.member);
                standard_json.errors.push(OutputError::new_error_with_data(
                    Some(file_id.as_str()),
                    error,
                    Some(SourceLocation::new(
                        file_id.to_string(),
                        text_range.start as isize,
                        text_range.end as isize,
                    )),
                    Some(input_sources),
                ));
            }
        }
        standard_json.benchmarks.extend(self.benchmarks);
    }
}
