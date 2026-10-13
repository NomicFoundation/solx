//!
//! The per-source symbol table for debuggers and stack tracers.
//!

pub mod contract;
pub mod function;
pub mod ranges;

use slang_solidity_v2::ast::SourceUnit;
use slang_solidity_v2::ast::SourceUnitMember;

use self::contract::Contract;
use self::function::Function;
use self::ranges::Ranges;

///
/// The per-source symbol table for debuggers and stack tracers.
///
/// Every range is `[byteOffset, byteLength]` into the source content.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugSymbols {
    /// The contracts, interfaces and libraries, in source order.
    pub contracts: Vec<Contract>,
    /// The free functions, in source order.
    pub free_functions: Vec<Function>,
    /// The ranges of the AST's sequence and collection nodes that cover text, sorted and
    /// deduplicated.
    pub ranges: Vec<[usize; 2]>,
}

impl From<&SourceUnit> for DebugSymbols {
    /// Collects declarations in source order and the ranges of nonterminal nodes.
    fn from(unit: &SourceUnit) -> Self {
        let mut contracts = Vec::new();
        let mut free_functions = Vec::new();
        for member in unit.members().iter() {
            match member {
                SourceUnitMember::ContractDefinition(contract) => {
                    contracts.push(Contract::from(&contract));
                }
                SourceUnitMember::InterfaceDefinition(interface) => {
                    contracts.push(Contract::from(&interface));
                }
                SourceUnitMember::LibraryDefinition(library) => {
                    contracts.push(Contract::from(&library));
                }
                SourceUnitMember::FunctionDefinition(function) => {
                    free_functions.push(Function::from(&function));
                }
                _ => {}
            }
        }
        Self {
            contracts,
            free_functions,
            ranges: Ranges::collect(unit),
        }
    }
}
