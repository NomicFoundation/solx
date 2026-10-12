//!
//! The per-source symbol table for debuggers and stack tracers.
//!

pub mod contract;
pub mod function;

use self::contract::Contract;
use self::function::Function;

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
