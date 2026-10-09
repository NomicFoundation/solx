//!
//! A contract, interface or library in the debug symbol table.
//!

use super::function::Function;

///
/// A contract, interface or library in the debug symbol table.
///
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contract {
    /// The declared name.
    pub name: String,
    /// Whether the definition is a contract, an interface or a library.
    pub kind: ContractKind,
    /// Whether the contract is declared `abstract`.
    #[serde(rename = "abstract")]
    pub is_abstract: bool,
    /// The `[byteOffset, byteLength]` of the whole definition.
    pub range: [usize; 2],
    /// The C3 linearisation, the definition itself first, leaving out a base that does not resolve.
    /// Present for contracts and interfaces.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bases: Option<Vec<Base>>,
    /// The members the definition itself declares, in declaration order.
    pub functions: Vec<Function>,
}

///
/// The kind of a definition in the debug symbol table.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContractKind {
    /// A contract, abstract or not.
    Contract,
    /// An interface.
    Interface,
    /// A library.
    Library,
}

///
/// A base in a linearisation, keyed by its standard JSON source path and name.
///
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Base {
    /// The standard JSON source path.
    pub path: String,
    /// The declared name of the definition in `path`.
    pub name: String,
}
