//!
//! Super / C3 virtual-dispatch precompute pass.
//!
//! Re-resolves a contract's `super.f(...)` and virtual internal calls against
//! its C3 linearisation, producing the `redirect` / `shadowed` /
//! `virtual_redirect` maps the frozen [`Context`](solx_mlir::Context) carries.
//!

use std::collections::HashMap;
use std::collections::HashSet;

use slang_solidity_v2::ast::ContractBase;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::visitor::Visitor;
use slang_solidity_v2::ast::visitor::accept_function_definition;

use crate::ast::ExpressionExt;
use crate::ast::contract::function::FunctionEmitter;

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
        let mro: Vec<ContractDefinition> = contract
            .linearised_bases()
            .into_iter()
            .filter_map(|base| match base {
                ContractBase::Contract(base_contract) => Some(base_contract),
                ContractBase::Interface(_) => None,
            })
            .collect();
        let most_derived_ids: HashSet<NodeId> = contract
            .linearised_functions()
            .iter()
            .map(|function| function.node_id())
            .collect();

        let mut dispatch = SuperDispatch::default();

        // Virtual dispatch: map every shadowed (overridden) base function to the
        // most-derived implementation of its signature, so a plain `g()` call in
        // a base body reaches the override. The most-derived version of each
        // signature is exactly the one kept by `linearised_functions`.
        let mut most_derived_by_signature: HashMap<String, NodeId> = HashMap::new();
        for function in contract.linearised_functions() {
            most_derived_by_signature
                .entry(FunctionEmitter::mlir_function_name(&function))
                .or_insert_with(|| function.node_id());
        }
        for base_contract in &mro {
            for function in base_contract.functions() {
                let node_id = function.node_id();
                if most_derived_ids.contains(&node_id) {
                    continue;
                }
                if let Some(&target) =
                    most_derived_by_signature.get(&FunctionEmitter::mlir_function_name(&function))
                    && target != node_id
                {
                    dispatch.virtual_redirect.insert(node_id, target);
                }
            }
        }

        let mut shadowed_ids: HashSet<NodeId> = HashSet::new();
        let mut walked: HashSet<NodeId> = HashSet::new();

        // Seed with every function the most-derived contract actually runs (its
        // linearised functions and the constructors along the chain), each tagged
        // with the mro index of the contract whose body it is.
        let mut to_walk: Vec<(usize, FunctionDefinition)> = Vec::new();
        for function in contract.linearised_functions() {
            let index = Self::defining_index(&mro, function.node_id()).unwrap_or(0);
            to_walk.push((index, function));
        }
        for (index, base_contract) in mro.iter().enumerate() {
            if let Some(constructor) = base_contract.constructor() {
                to_walk.push((index, constructor));
            }
        }

        while let Some((from_index, function)) = to_walk.pop() {
            if !walked.insert(function.node_id()) {
                continue;
            }
            // Gather this function's `super.f` / `Base.f` accesses (the visitor
            // populates the consolidated working fields on a fresh instance).
            let mut collector = SuperDispatch::default();
            accept_function_definition(&function, &mut collector);
            for (access_id, lexical_target) in collector.super_calls {
                let signature = FunctionEmitter::mlir_function_name(&lexical_target);
                let Some((target_index, target)) =
                    Self::resolve_super_target(&mro, from_index, &signature)
                else {
                    continue;
                };
                Self::record_target(
                    &mut dispatch,
                    &mut shadowed_ids,
                    &mut to_walk,
                    &mro,
                    access_id,
                    target_index,
                    target,
                    &most_derived_ids,
                );
            }
            for (access_id, target) in collector.base_calls {
                // `Base.f` names the override exactly; its defining contract gives
                // the qualified symbol and the index to walk from.
                let Some(target_index) = Self::defining_index(&mro, target.node_id()) else {
                    continue;
                };
                Self::record_target(
                    &mut dispatch,
                    &mut shadowed_ids,
                    &mut to_walk,
                    &mro,
                    access_id,
                    target_index,
                    target,
                    &most_derived_ids,
                );
            }
        }

        dispatch
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

    /// Finds the `super` target of `signature` for a call in a function defined
    /// at `from_index`: the first *implemented* function with that signature in a
    /// strictly more-base contract (`mro[from_index + 1 ..]`). An unimplemented
    /// (bodyless) interface / abstract declaration is skipped, like solc.
    fn resolve_super_target(
        mro: &[ContractDefinition],
        from_index: usize,
        signature: &str,
    ) -> Option<(usize, FunctionDefinition)> {
        for (index, contract) in mro.iter().enumerate().skip(from_index + 1) {
            for function in contract.functions() {
                if function.body().is_some()
                    && FunctionEmitter::mlir_function_name(&function) == signature
                {
                    return Some((index, function));
                }
            }
        }
        None
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
        dispatch.redirect.insert(access_id, target.node_id());
        if !most_derived_ids.contains(&target.node_id()) && shadowed_ids.insert(target.node_id()) {
            let symbol = format!(
                "{}.{}",
                mro[target_index].name().name(),
                FunctionEmitter::mlir_function_name(&target)
            );
            dispatch.shadowed.push((symbol, target.clone()));
        }
        to_walk.push((target_index, target));
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
