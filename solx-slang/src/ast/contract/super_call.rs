//!
//! Resolution of `super` calls against the C3 linearisation.
//!
//! `super.f(...)` dispatches to the next implementation of `f` up the C3
//! linearisation of the *most-derived* contract being compiled — not the
//! contract in which the call lexically appears. In a diamond (`D is B, C`,
//! both `is A`) `C`'s `super.f()` must reach `B.f` when `D` is deployed, even
//! though `C` alone would reach `A.f`. Slang's binder resolves `super.f`
//! lexically, so it cannot see `B` from inside `C`.
//!
//! This module re-resolves every `super` call against the most-derived
//! contract's linearised bases and produces:
//!   * a redirect map (`super` member-access node id -> target function node
//!     id) consumed at the call site, and
//!   * the shadowed base overrides reached this way, paired with a
//!     contract-qualified, selector-less symbol, so the contract emitter can
//!     register and emit them as distinct internal functions.
//!
//! Targets are matched by signature ([`FunctionEmitter::mlir_function_name`])
//! and must be *implemented* (have a body), so an unimplemented interface /
//! abstract declaration is skipped exactly like solc's `super`.

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

use crate::ast::contract::function::FunctionEmitter;

/// The result of re-resolving a contract's `super` and virtual calls against
/// its C3 linearisation.
#[derive(Default)]
pub(super) struct SuperDispatch {
    /// `super` member-access node id -> target function node id.
    pub redirect: HashMap<NodeId, NodeId>,
    /// Shadowed base overrides reached through `super`, paired with the
    /// contract-qualified symbol they must be emitted under (deduplicated by
    /// node id). These are emitted internal-only (no selector).
    pub shadowed: Vec<(String, FunctionDefinition)>,
    /// Virtual dispatch: a plain internal call resolving (lexically) to an
    /// overridden base function must reach the most-derived override. Maps each
    /// shadowed base function node id -> the most-derived function node id of
    /// the same signature. Unlike `super`, no extra emission is needed (the
    /// most-derived version is already emitted with its selector).
    pub virtual_redirect: HashMap<NodeId, NodeId>,
}

/// Visitor that records `super.f` and explicit base-call (`Base.f`) member
/// accesses (the access node and the function its member resolves to).
#[derive(Default)]
struct SuperCallCollector {
    /// `super.f` accesses — the recorded function is the *lexical* resolution,
    /// used only for its signature (the real target is re-resolved against the
    /// most-derived linearisation).
    super_calls: Vec<(NodeId, FunctionDefinition)>,
    /// `Base.f` accesses where `Base` is a named contract — the recorded
    /// function is the exact target (slang resolves the named base correctly,
    /// even in a diamond).
    base_calls: Vec<(NodeId, FunctionDefinition)>,
}

impl Visitor for SuperCallCollector {
    fn enter_member_access_expression(&mut self, node: &MemberAccessExpression) -> bool {
        // Peel parenthesisation around the operand: `(super).f` / `(Base).f`
        // are the same internal calls as `super.f` / `Base.f`.
        let operand = unwrap_parens(node.operand());
        if matches!(operand, Expression::SuperKeyword(_)) {
            if let Some(Definition::Function(function)) = node.member().resolve_to_definition() {
                self.super_calls.push((node.node_id(), function));
            }
        } else if let Expression::Identifier(identifier) = &operand
            && matches!(identifier.resolve_to_definition(), Some(Definition::Contract(_)))
            && let Some(Definition::Function(function)) = node.member().resolve_to_definition()
        {
            // `Base.f(...)` — an explicit base-qualified internal call.
            self.base_calls.push((node.node_id(), function));
        }
        // Descend so nested calls (e.g. `super.f(super.g())`) are found.
        true
    }
}

/// Peels parenthesisation (single-element tuples) from an expression, so a
/// parenthesised receiver (`(super).f`, `(Base).f`) is treated like the bare
/// form.
fn unwrap_parens(mut expression: Expression) -> Expression {
    loop {
        let inner = match &expression {
            Expression::TupleExpression(tuple) if tuple.items().len() == 1 => {
                tuple.items().iter().next().and_then(|item| item.expression())
            }
            _ => None,
        };
        match inner {
            Some(next) => expression = next,
            None => return expression,
        }
    }
}

/// Returns the mro index of the contract that lexically defines `node_id`
/// (the contract whose own `functions()` contains it), if any.
fn defining_index(mro: &[ContractDefinition], node_id: NodeId) -> Option<usize> {
    mro.iter().position(|contract| {
        contract
            .functions()
            .iter()
            .any(|function| function.node_id() == node_id)
    })
}

/// Finds the `super` target of signature `signature` for a call appearing in a
/// function defined at `from_index`: the first *implemented* function with that
/// signature in a strictly more-base contract (`mro[from_index + 1 ..]`).
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

/// Re-resolves every `super` call reachable from `contract`'s functions and
/// constructor against its C3 linearisation.
pub(super) fn build_super_dispatch(contract: &ContractDefinition) -> SuperDispatch {
    let mro: Vec<ContractDefinition> = contract
        .compute_linearised_bases()
        .into_iter()
        .filter_map(|base| match base {
            ContractBase::Contract(base_contract) => Some(base_contract),
            ContractBase::Interface(_) => None,
        })
        .collect();
    let most_derived_ids: HashSet<NodeId> = contract
        .compute_linearised_functions()
        .iter()
        .map(|function| function.node_id())
        .collect();

    let mut dispatch = SuperDispatch::default();

    // Virtual dispatch: map every shadowed (overridden) base function to the
    // most-derived implementation of its signature, so a plain `g()` call in a
    // base body reaches the override. The most-derived version of each
    // signature is exactly the one kept by `compute_linearised_functions`.
    let mut most_derived_by_signature: HashMap<String, NodeId> = HashMap::new();
    for function in contract.compute_linearised_functions() {
        most_derived_by_signature
            .entry(FunctionEmitter::mlir_function_name(&function))
            .or_insert_with(|| function.node_id());
    }
    for base_contract in &mro {
        for function in base_contract.functions() {
            let node_id = function.node_id();
            if function.body().is_none() || most_derived_ids.contains(&node_id) {
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
    for function in contract.compute_linearised_functions() {
        let index = defining_index(&mro, function.node_id()).unwrap_or(0);
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
        let mut collector = SuperCallCollector::default();
        accept_function_definition(&function, &mut collector);
        for (access_id, lexical_target) in collector.super_calls {
            let signature = FunctionEmitter::mlir_function_name(&lexical_target);
            let Some((target_index, target)) =
                resolve_super_target(&mro, from_index, &signature)
            else {
                continue;
            };
            record_target(
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
            // `Base.f` names the override exactly; use it directly. Its defining
            // contract gives the qualified symbol and the index to walk from.
            let Some(target_index) = defining_index(&mro, target.node_id()) else {
                continue;
            };
            record_target(
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

/// Records a `super`/base-call target: adds the redirect, registers it as a
/// shadowed override (with a contract-qualified symbol) when it is not the
/// most-derived version, and enqueues its body for further `super`/base calls.
#[allow(clippy::too_many_arguments)]
fn record_target(
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
