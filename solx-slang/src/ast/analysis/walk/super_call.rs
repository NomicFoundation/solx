//!
//! Super / C3 virtual-dispatch precompute pass.
//!
//! Re-resolves a contract's `super.f(...)` and virtual internal calls against its C3 linearisation,
//! producing the `redirect` / `shadowed` / `virtual_redirect` maps contract emission carries.
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
use slang_solidity_v2::ast::visitor::accept_inheritance_type;

use crate::ast::contract::function::FunctionEmitter;

/// The result of re-resolving a contract's `super` / virtual calls against its C3 linearisation,
/// plus the pass-local visitor state gathering the `super.f` / `Base.f` accesses.
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
    /// Working state: the `super.f` accesses gathered by the visitor.
    super_calls: Vec<(NodeId, FunctionDefinition)>,
    /// Working state: the `Base.f` accesses gathered by the visitor.
    base_calls: Vec<(NodeId, FunctionDefinition)>,
}

impl SuperDispatch {
    /// Re-resolves every `super` call reachable from `contract`'s functions and
    /// constructor against its C3 linearisation. Base-qualified calls in inheritance-specifier
    /// arguments (`contract C is Base(Other.f())`), which no function body contains, are collected
    /// too so each gets a `redirect` entry.
    pub fn build_super_dispatch(contract: &ContractDefinition) -> Self {
        let mro: Vec<ContractDefinition> = contract
            .linearised_bases()
            .into_iter()
            .filter_map(|base| match base {
                ContractBase::Contract(base_contract) => Some(base_contract),
                ContractBase::Interface(_) => None,
            })
            .collect();
        let linearised_functions = contract.linearised_functions();
        let most_derived_ids: HashSet<NodeId> = linearised_functions
            .iter()
            .map(|function| function.node_id())
            .collect();

        let mut dispatch = Self::default();

        for base_contract in mro.iter() {
            for function in base_contract.functions() {
                let node_id = function.node_id();
                if most_derived_ids.contains(&node_id) {
                    continue;
                }
                if let Some(target) = linearised_functions
                    .iter()
                    .find(|most_derived| most_derived.overrides(&function))
                    && target.node_id() != node_id
                {
                    dispatch.virtual_redirect.insert(node_id, target.node_id());
                }
            }
        }

        let mut shadowed_ids: HashSet<NodeId> = HashSet::new();
        let mut walked: HashSet<NodeId> = HashSet::new();

        let mut to_walk: Vec<(usize, FunctionDefinition)> = Vec::new();
        for function in contract.linearised_functions() {
            let index = Self::defining_index(&mro, function.node_id())
                .expect("a linearised function is defined by a contract in the mro");
            to_walk.push((index, function));
        }
        for (index, base_contract) in mro.iter().enumerate() {
            if let Some(constructor) = base_contract.constructor() {
                to_walk.push((index, constructor));
            }
        }

        for base_contract in mro.iter() {
            let mut collector = Self::default();
            for inheritance in base_contract.inheritance_types().iter() {
                accept_inheritance_type(&inheritance, &mut collector);
            }
            for (access_id, target) in collector.base_calls {
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

        while let Some((from_index, function)) = to_walk.pop() {
            if !walked.insert(function.node_id()) {
                continue;
            }
            let mut collector = Self::default();
            accept_function_definition(&function, &mut collector);
            for (access_id, lexical_target) in collector.super_calls {
                let (target_index, target) =
                    Self::resolve_super_target(&mro, from_index, &lexical_target)
                        .expect("slang validated: a super call resolves to an implemented base");
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
    fn defining_index(mro: &[ContractDefinition], node_id: NodeId) -> Option<usize> {
        mro.iter().position(|contract| {
            contract
                .functions()
                .iter()
                .any(|function| function.node_id() == node_id)
        })
    }

    /// Finds the `super` target for `lexical_target` at `from_index`: the first *implemented*
    /// function it overrides in a strictly more-base contract; a bodyless declaration is skipped.
    fn resolve_super_target(
        mro: &[ContractDefinition],
        from_index: usize,
        lexical_target: &FunctionDefinition,
    ) -> Option<(usize, FunctionDefinition)> {
        for (index, contract) in mro.iter().enumerate().skip(from_index + 1) {
            for function in contract.functions() {
                if function.body().is_some() && lexical_target.overrides(&function) {
                    return Some((index, function));
                }
            }
        }
        None
    }

    /// Records one resolved super/base target into the dispatch maps,
    /// scheduling any shadowed override for emission under its
    /// contract-qualified symbol.
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
}

impl Visitor for SuperDispatch {
    fn enter_member_access_expression(&mut self, node: &MemberAccessExpression) -> bool {
        let operand = node.operand().unwrap_parentheses();
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
            self.base_calls.push((node.node_id(), function));
        }
        true
    }
}
