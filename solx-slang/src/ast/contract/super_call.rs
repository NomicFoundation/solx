//!
//! Collection of base-override functions reached through `super`.
//!
//! `super.f(...)` dispatches to the next implementation of `f` up the C3
//! linearisation, skipping the current contract's own override. When the
//! target is a *non-overridden* inherited function it is already part of the
//! linearised set (and emitted with its selector); when it is a *shadowed*
//! override (a base version that a more-derived contract overrides) it is
//! filtered out of `compute_linearised_functions` and never emitted. This
//! module walks a contract's functions (transitively through the `super`
//! targets they reach) and returns the node ids of every function reached via
//! `super`, so the contract emitter can register and emit the shadowed ones
//! under contract-qualified, selector-less symbols.

use std::collections::HashSet;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::visitor::Visitor;
use slang_solidity_v2::ast::visitor::accept_function_definition;

/// Visitor that records every `super.f` member access whose member resolves to
/// a function definition.
#[derive(Default)]
struct SuperCallCollector {
    targets: Vec<FunctionDefinition>,
}

impl Visitor for SuperCallCollector {
    fn enter_member_access_expression(&mut self, node: &MemberAccessExpression) -> bool {
        if matches!(node.operand(), Expression::SuperKeyword(_))
            && let Some(Definition::Function(function)) = node.member().resolve_to_definition()
        {
            self.targets.push(function);
        }
        // Descend so nested `super` calls (e.g. `super.f(super.g())`) are found.
        true
    }
}

/// Returns the node ids of every function reached through `super` from
/// `contract`'s functions, including those reached only through other
/// `super`-targeted functions (a base override whose own body calls `super`).
pub(super) fn collect_super_target_ids(contract: &ContractDefinition) -> HashSet<NodeId> {
    let mut collected: HashSet<NodeId> = HashSet::new();
    let mut walked: HashSet<NodeId> = HashSet::new();
    let mut to_walk: Vec<FunctionDefinition> = contract.compute_linearised_functions();

    while let Some(function) = to_walk.pop() {
        if !walked.insert(function.node_id()) {
            continue;
        }
        let mut collector = SuperCallCollector::default();
        accept_function_definition(&function, &mut collector);
        for target in collector.targets {
            if collected.insert(target.node_id()) {
                // Newly seen — walk its body too so a `super`-reached override
                // that itself calls `super` pulls in the next base version.
                to_walk.push(target);
            }
        }
    }
    collected
}
