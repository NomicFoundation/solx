//!
//! Super / C3 virtual-dispatch precompute pass.
//!
//! Re-resolves a contract's `super.f(...)` and virtual internal calls against
//! its C3 linearisation, producing the `redirect` / `shadowed` /
//! `virtual_redirect` maps the frozen [`Context`](solx_mlir::Context) carries.
//!

use std::collections::HashMap;
use std::collections::HashSet;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::visitor::Visitor;

use crate::ast::ExpressionExt;

/// The result of re-resolving a contract's `super` and virtual calls against
/// its C3 linearisation, plus the pass-local visitor state that gathers the
/// `super.f` / `Base.f` accesses.
///
/// The SOLE top-level type of this module (§2a): the oracle's separate
/// `SuperCallCollector` visitor is consolidated onto this type (the `super_calls`
/// / `base_calls` working fields) so the module declares exactly one type.
#[derive(Default)]
pub struct SuperDispatch {
    /// `super` member-access node id -> target function node id.
    pub redirect: HashMap<NodeId, NodeId>,
    /// Shadowed base overrides reached through `super`, paired with the
    /// contract-qualified symbol they must be emitted under.
    pub shadowed: Vec<(String, FunctionDefinition)>,
    /// Virtual dispatch: shadowed base function node id -> most-derived override
    /// node id of the same signature.
    pub virtual_redirect: HashMap<NodeId, NodeId>,
    /// `super.f` accesses gathered by the visitor (working state).
    super_calls: Vec<(NodeId, FunctionDefinition)>,
    /// `Base.f` accesses gathered by the visitor (working state).
    base_calls: Vec<(NodeId, FunctionDefinition)>,
}

impl SuperDispatch {
    /// Re-resolves every `super` call reachable from `contract`'s functions and
    /// constructor against its C3 linearisation.
    pub fn build_super_dispatch(contract: &ContractDefinition) -> SuperDispatch {
        let _ = contract;
        unimplemented!("super/C3 dispatch precompute")
    }

    /// Returns the MRO index of the contract that lexically defines `node_id`.
    pub fn defining_index(mro: &[ContractDefinition], node_id: NodeId) -> Option<usize> {
        mro.iter().position(|contract| {
            contract
                .functions()
                .iter()
                .any(|function| function.node_id() == node_id)
        })
    }

    /// Records one resolved super/base target into the dispatch maps,
    /// scheduling the shadowed override (if any) for emission under its
    /// contract-qualified symbol. (Eight args sit at the `clippy.toml`
    /// `too-many-arguments-threshold`, so no `#[allow]` is needed — D2.)
    pub fn record_target(
        dispatch: &mut SuperDispatch,
        shadowed_ids: &mut HashSet<NodeId>,
        to_walk: &mut Vec<(usize, FunctionDefinition)>,
        mro: &[ContractDefinition],
        access_id: NodeId,
        target_index: usize,
        target: FunctionDefinition,
        most_derived_ids: &HashSet<NodeId>,
    ) {
        let _ = (
            dispatch,
            shadowed_ids,
            to_walk,
            mro,
            access_id,
            target_index,
            target,
            most_derived_ids,
        );
        unimplemented!("super target recording")
    }
}

impl Visitor for SuperDispatch {
    fn enter_member_access_expression(&mut self, node: &MemberAccessExpression) -> bool {
        // Peel parenthesisation: `(super).f` / `(Base).f` are the same internal
        // calls as `super.f` / `Base.f`.
        let operand = node.operand().unwrap_parens();
        if matches!(operand, Expression::SuperKeyword(_)) {
            if let Some(Definition::Function(function)) = node.member().resolve_to_definition() {
                self.super_calls.push((node.node_id(), function));
            }
        } else if let Expression::Identifier(identifier) = &operand
            && matches!(
                identifier.resolve_to_definition(),
                Some(Definition::Contract(_))
            )
            && let Some(Definition::Function(function)) = node.member().resolve_to_definition()
        {
            // `Base.f(...)` — an explicit base-qualified internal call.
            self.base_calls.push((node.node_id(), function));
        }
        // Descend so nested calls (e.g. `super.f(super.g())`) are found.
        true
    }
}
