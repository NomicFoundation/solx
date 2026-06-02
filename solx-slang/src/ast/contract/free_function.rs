//!
//! Collection of free functions referenced by a contract.
//!
//! Free functions (file-level `function f(...) {...}` declared outside any
//! contract) are not part of any contract's linearised function set, so the
//! per-contract MLIR module does not emit them unless a contract calls one.
//! This module walks a contract's functions (transitively through the free
//! functions they reach) and returns every free function called by name
//! (`f(...)`), so the contract emitter can pre-register and emit them into the
//! contract body, where they lower like ordinary internal functions.

use std::collections::HashMap;
use std::collections::HashSet;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::visitor::Visitor;
use slang_solidity_v2::ast::visitor::accept_function_definition;

/// Visitor that records every `f(...)` call whose callee is a plain identifier
/// resolving to one of the source unit's free functions.
struct FreeCallCollector<'a> {
    /// Node ids of the source unit's free functions to match call callees against.
    free_ids: &'a HashSet<NodeId>,
    /// The free functions reached by the visited calls, in discovery order.
    reached: Vec<FunctionDefinition>,
}

impl Visitor for FreeCallCollector<'_> {
    fn enter_function_call_expression(&mut self, node: &FunctionCallExpression) -> bool {
        if let Expression::Identifier(identifier) = node.operand()
            && let Some(Definition::Function(function)) = identifier.resolve_to_definition()
            && self.free_ids.contains(&function.node_id())
        {
            self.reached.push(function);
        }
        // Descend so nested calls (e.g. `f(g(x))`) are also recorded.
        true
    }
}

/// Returns the free functions reachable from `contract`'s own functions and
/// constructor, including those reached only through other free functions.
/// `free_functions` is the source unit's full set of free functions.
/// `extra_roots` are additional function bodies to walk that are not part of
/// the linearised set — the shadowed base overrides reached only through
/// `super` (which are emitted into this module and may call free functions of
/// their own). The result is deduplicated by node id.
pub(super) fn collect_free_functions(
    contract: &ContractDefinition,
    free_functions: &[FunctionDefinition],
    extra_roots: &[FunctionDefinition],
) -> Vec<FunctionDefinition> {
    let free_ids: HashSet<NodeId> = free_functions.iter().map(|f| f.node_id()).collect();
    if free_ids.is_empty() {
        return Vec::new();
    }

    let mut collected: HashMap<NodeId, FunctionDefinition> = HashMap::new();
    let mut walked: HashSet<NodeId> = HashSet::new();
    let mut to_walk: Vec<FunctionDefinition> = contract.compute_linearised_functions();
    if let Some(constructor) = contract.constructor() {
        to_walk.push(constructor);
    }
    to_walk.extend(extra_roots.iter().cloned());

    while let Some(function) = to_walk.pop() {
        if !walked.insert(function.node_id()) {
            continue;
        }
        let mut collector = FreeCallCollector {
            free_ids: &free_ids,
            reached: Vec::new(),
        };
        accept_function_definition(&function, &mut collector);
        for free_function in collector.reached {
            if collected
                .insert(free_function.node_id(), free_function.clone())
                .is_none()
            {
                // Newly seen — walk its body too, in case it reaches further
                // free functions.
                to_walk.push(free_function);
            }
        }
    }
    collected.into_values().collect()
}
