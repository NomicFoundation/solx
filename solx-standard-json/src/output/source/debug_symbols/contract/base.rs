//!
//! A base in a linearisation.
//!

use slang_solidity_v2::ast::ContractBase;

///
/// A base in a linearisation, keyed by its standard JSON source path and name.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Base {
    /// The standard JSON source path.
    pub path: String,
    /// The declared name of the definition in `path`.
    pub name: String,
}

impl From<&ContractBase> for Base {
    /// Uses the definition's source path to identify bases across files.
    fn from(base: &ContractBase) -> Self {
        match base {
            ContractBase::Contract(contract) => Self {
                path: contract.get_file_id().as_str().to_owned(),
                name: contract.name().name().to_owned(),
            },
            ContractBase::Interface(interface) => Self {
                path: interface.get_file_id().as_str().to_owned(),
                name: interface.name().name().to_owned(),
            },
        }
    }
}
