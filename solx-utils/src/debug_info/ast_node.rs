//!
//! solc AST node.
//!

use crate::debug_info::IDebugInfoAstNode;
use crate::debug_info::mapped_location::MappedLocation;
use crate::debug_info::solc_location::SolcLocation;

///
/// solc AST node.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AstNode {
    /// AST ID.
    pub ast_id: usize,
    /// solc-style location.
    pub solc_location: SolcLocation,
    /// Line-number-style location.
    pub mapped_location: MappedLocation,
}

impl AstNode {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        ast_id: usize,
        solc_location: SolcLocation,
        mapped_location: MappedLocation,
    ) -> Self {
        Self {
            ast_id,
            solc_location,
            mapped_location,
        }
    }
}

impl IDebugInfoAstNode for AstNode {
    type Key = usize;

    fn index_id(&self) -> Self::Key {
        assert!(
            self.solc_location.start >= 0,
            "The final stop for potential -1 values"
        );

        self.solc_location.start as usize
    }
}
