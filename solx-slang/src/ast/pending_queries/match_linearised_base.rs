//!
//! `is Base` / base-constructor path resolution against the C3 linearisation (pure-Slang, pending a home).
//!

use std::collections::HashSet;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::IdentifierPath;
use slang_solidity_v2::ast::NodeId;

/// Resolves an `is Base` / base-constructor `Base(args)` path reference to its
/// contract in the C3 linearisation.
pub trait MatchLinearisedBase {
    /// The `mro` entry this path names (the whole path `Base`, else its final segment `M.Base`),
    /// or `None` if it does not resolve to a linearised base.
    fn match_linearised_base(
        &self,
        mro: &[ContractDefinition],
        mro_node_ids: &HashSet<NodeId>,
    ) -> Option<ContractDefinition>;
}

impl MatchLinearisedBase for IdentifierPath {
    fn match_linearised_base(
        &self,
        mro: &[ContractDefinition],
        mro_node_ids: &HashSet<NodeId>,
    ) -> Option<ContractDefinition> {
        let base_definition = self
            .resolve_to_definition()
            .or_else(|| self.iter().last()?.resolve_to_definition());
        let Some(Definition::Contract(base_contract)) = base_definition else {
            return None;
        };
        if !mro_node_ids.contains(&base_contract.node_id()) {
            return None;
        }
        mro.iter()
            .find(|contract| contract.node_id() == base_contract.node_id())
            .cloned()
    }
}
