//!
//! The `solc --standard-json` output source.
//!

pub mod debug_symbols;

use serde_json::value::RawValue;

use self::debug_symbols::DebugSymbols;

///
/// The `solc --standard-json` output source.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    /// Source code ID.
    pub id: usize,
    /// Source code AST.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ast: Option<Box<RawValue>>,
    /// The symbol table for debuggers and stack tracers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debug_symbols: Option<DebugSymbols>,
}

impl Source {
    ///
    /// Initializes a standard JSON source.
    ///
    /// Is used for projects compiled without `solc`.
    ///
    pub fn new(id: usize) -> Self {
        Self {
            id,
            ast: None,
            debug_symbols: None,
        }
    }
}
