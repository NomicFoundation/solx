//!
//! A contract, interface or library in the debug symbol table.
//!

pub mod base;

use super::function::Function;

use self::base::Base;

///
/// A contract, interface or library in the debug symbol table.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Contract {
    /// The declared name.
    pub name: String,
    /// Whether the definition is a contract, an interface or a library.
    pub kind: solx_mlir::ContractKind,
    /// Whether the contract is declared `abstract`.
    pub r#abstract: bool,
    /// The `[byteOffset, byteLength]` of the whole definition.
    pub range: [usize; 2],
    /// The C3 linearisation, the definition itself first. Present for contracts and interfaces.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bases: Option<Vec<Base>>,
    /// The members the definition itself declares, in declaration order.
    pub functions: Vec<Function>,
}
