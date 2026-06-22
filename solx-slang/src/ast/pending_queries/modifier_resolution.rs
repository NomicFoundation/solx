//!
//! Modifier C3-resolution queries (pure-Slang, pending a permanent home).
//!

use std::collections::HashMap;

use slang_solidity_v2::ast::ContractBase;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::ModifierInvocation;

/// Resolves a modifier invocation against the contract's C3-linearised modifier
/// set — virtual override re-dispatch and namespace-qualified resolution.
pub trait ModifierResolution {
    /// Every modifier across the contract's C3-linearised bases (most-derived
    /// first).
    fn linearised_modifiers(&self) -> Vec<FunctionDefinition>;

    /// The most-derived body-bearing modifier per name, across the C3 linearisation (modifiers can't
    /// be overloaded, so the name uniquely keys an override chain).
    fn most_derived_modifiers_by_name(&self) -> HashMap<String, FunctionDefinition>;

    /// Re-dispatches a virtual modifier invocation to its most-derived body-bearing implementation.
    /// `None` (keep lexical resolution) for a qualified `Base.m` or a modifier outside this hierarchy.
    fn resolve_modifier_override(
        &self,
        invocation: &ModifierInvocation,
        resolved: &FunctionDefinition,
    ) -> Option<FunctionDefinition>;

    /// Resolves a qualified modifier invocation by last-segment name; `None` for a base-constructor invocation.
    fn resolve_qualified_modifier(
        &self,
        invocation: &ModifierInvocation,
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

    fn most_derived_modifiers_by_name(&self) -> HashMap<String, FunctionDefinition> {
        let mut by_name: HashMap<String, FunctionDefinition> = HashMap::new();
        for modifier in self.linearised_modifiers() {
            if modifier.body().is_none() {
                continue;
            }
            let Some(name) = modifier.name().map(|identifier| identifier.name()) else {
                continue;
            };
            by_name.entry(name).or_insert(modifier);
        }
        by_name
    }

    fn resolve_modifier_override(
        &self,
        invocation: &ModifierInvocation,
        resolved: &FunctionDefinition,
    ) -> Option<FunctionDefinition> {
        if invocation.name().len() > 1 {
            return None;
        }
        let resolved_id = resolved.node_id();
        if !self
            .linearised_modifiers()
            .iter()
            .any(|modifier| modifier.node_id() == resolved_id)
        {
            return None;
        }
        let name = resolved.name().map(|identifier| identifier.name())?;
        self.most_derived_modifiers_by_name().get(&name).cloned()
    }

    fn resolve_qualified_modifier(
        &self,
        invocation: &ModifierInvocation,
    ) -> Option<FunctionDefinition> {
        let modifier_name = invocation.name().iter().last()?.name();
        self.most_derived_modifiers_by_name()
            .get(&modifier_name)
            .cloned()
    }
}
