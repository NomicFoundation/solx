//!
//! TODO: pure-Slang query pending a home (Slang dev-solx vs solx vs fold) —
//! query-sorting pass. Lifted verbatim from `FunctionEmitter`'s modifier
//! C3-resolution methods.
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

    /// The most-derived modifier with a body, per name, across the contract's C3
    /// linearisation. Modifiers cannot be overloaded, so the name uniquely keys an
    /// override chain; `linearised_bases` is most-derived first, so the first
    /// body-bearing modifier of each name is the active override. The name is only
    /// ever a map key — never string-compared.
    fn most_derived_modifiers_by_name(&self) -> HashMap<String, FunctionDefinition>;

    /// Re-dispatches a virtual modifier invocation to its most-derived
    /// implementation with a body (qualified invocations resolve directly).
    ///
    /// Returns `None` — keep the lexical resolution — when the invocation is
    /// qualified (`Base.m`, which names a specific modifier and bypasses virtual
    /// dispatch) or when the resolved modifier is not part of this contract's
    /// hierarchy (e.g. a library modifier reached through `using L for *`, which
    /// must not be virtual-dispatched against a same-named modifier of the using
    /// contract).
    fn resolve_modifier_override(
        &self,
        invocation: &ModifierInvocation,
        resolved: &FunctionDefinition,
    ) -> Option<FunctionDefinition>;

    /// Resolves a qualified modifier invocation by last-segment name against the
    /// C3 modifiers; `None` marks a base-constructor invocation, whose final
    /// segment is a contract name.
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
