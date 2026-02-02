//!
//! The LLVM IR Solidity data trait.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;

///
/// The LLVM IR Solidity data trait.
///
pub trait ISolidityData {
    ///
    /// Returns all runtime code offsets for the specified `id`.
    ///
    fn immutable_offsets(&mut self, id: &str) -> Option<BTreeSet<u64>>;

    ///
    /// Returns the Solidity source code file IDs and paths used in the translation unit.
    ///
    fn sources(&self) -> &BTreeMap<usize, String>;

    ///
    /// Returns the AST contract definition by its name.
    ///
    fn debug_info_contract_definition(&self) -> Option<&solx_utils::DebugInfoContractDefinition>;

    ///
    /// Returns the AST function definition by its AST node ID.
    ///
    fn debug_info_function_definition(
        &self,
        node_id: usize,
    ) -> Option<&solx_utils::DebugInfoFunctionDefinition>;

    ///
    /// Sets the current Solidity source code location.
    ///
    fn set_debug_info_solc_location(&mut self, solc_location: solx_utils::DebugInfoSolcLocation);

    ///
    /// Gets the current solc-style source code location.
    ///
    fn get_debug_info_solc_location(&self) -> Option<&solx_utils::DebugInfoSolcLocation>;

    ///
    /// Gets the current solx-style source code location.
    ///
    fn get_solx_location(&self) -> Option<&solx_utils::DebugInfoMappedLocation>;

    ///
    /// Returns a reference to the Solidity AST debug info.
    ///
    fn debug_info(&self) -> Option<&solx_utils::DebugInfo>;
}
