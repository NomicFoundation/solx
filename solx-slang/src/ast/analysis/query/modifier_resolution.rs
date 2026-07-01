//!
//! Modifier C3-resolution queries (pure-Slang).
//!

use slang_solidity_v2::ast::ContractBase;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::ModifierInvocation;

/// Resolves a modifier invocation against the contract's C3-linearised modifier set: virtual override
/// re-dispatch.
pub trait ModifierResolution {
    /// Every modifier across the contract's C3-linearised bases (most-derived first).
    fn linearised_modifiers(&self) -> Vec<FunctionDefinition>;

    /// Re-dispatches a virtual modifier invocation to its most-derived body-bearing override, `None`
    /// to keep lexical resolution: a qualified `Base.m` or a modifier outside this hierarchy.
    fn resolve_modifier_override(
        &self,
        invocation: &ModifierInvocation,
        resolved: &FunctionDefinition,
    ) -> Option<FunctionDefinition>;
}

impl ModifierResolution for ContractDefinition {
    fn linearised_modifiers(&self) -> Vec<FunctionDefinition> {
        self.linearised_bases()
            .into_iter()
            .filter_map(|base| match base {
                ContractBase::Contract(base_contract) => Some(base_contract),
                ContractBase::Interface(_) => None,
            })
            .flat_map(|base_contract| base_contract.modifiers())
            .collect()
    }

    fn resolve_modifier_override(
        &self,
        invocation: &ModifierInvocation,
        resolved: &FunctionDefinition,
    ) -> Option<FunctionDefinition> {
        if invocation.name().len() > 1 {
            return None;
        }
        let linearised = self.linearised_modifiers();
        if !linearised
            .iter()
            .any(|modifier| modifier.node_id() == resolved.node_id())
        {
            return None;
        }
        linearised
            .into_iter()
            .find(|modifier| modifier.body().is_some() && modifier.overrides(resolved))
    }
}
