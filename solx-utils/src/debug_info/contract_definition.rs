//!
//! solc AST contract definition.
//!

use crate::debug_info::mapped_location::MappedLocation;
use crate::debug_info::solc_location::SolcLocation;
use crate::debug_info::IDebugInfoAstNode;

///
/// solc AST contract definition.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContractDefinition {
    /// Contract AST ID.
    pub ast_id: usize,
    /// Contract name.
    pub name: String,
    /// solc-style location.
    pub solc_location: SolcLocation,
    /// Line-number-style location.
    pub mapped_location: MappedLocation,
}

impl ContractDefinition {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        ast_id: usize,
        name: String,
        solc_location: SolcLocation,
        mapped_location: MappedLocation,
    ) -> Self {
        Self {
            ast_id,
            name,
            solc_location,
            mapped_location,
        }
    }
}

impl IDebugInfoAstNode for ContractDefinition {
    type Key = String;

    fn index_id(&self) -> Self::Key {
        self.name.clone()
    }
}
