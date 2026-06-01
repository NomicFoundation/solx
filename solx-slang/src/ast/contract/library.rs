//!
//! Collection of library functions referenced by a contract.
//!
//! The Slang frontend compiles one contract per MLIR module, so a library's
//! `internal` functions are not emitted unless a contract calls them. This
//! module walks a contract's functions (transitively through the library
//! functions they reach) and returns every directly-called library function
//! (`L.f(...)`) so the contract emitter can pre-register and emit them into the
//! contract body, where they lower like ordinary internal functions.

use std::collections::HashMap;
use std::collections::HashSet;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionVisibility;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::visitor::Visitor;
use slang_solidity_v2::ast::visitor::accept_function_definition;

/// Visitor that records every `L.f` member access whose base resolves to a
/// library and whose member resolves to a function.
#[derive(Default)]
struct LibraryCallCollector {
    functions: Vec<FunctionDefinition>,
}

impl Visitor for LibraryCallCollector {
    fn enter_member_access_expression(&mut self, node: &MemberAccessExpression) -> bool {
        // A member access whose member resolves to a library function with no
        // external selector is an internal library call — `internal`/`private`
        // functions, but also `public` ones with non-ABI-encodable parameters
        // (a `storage` / `mapping` argument), which solc calls internally
        // (passing the slot) rather than by `delegatecall`. Either way they are
        // inlined into the contract here. Library functions that *do* have a
        // selector are reached by delegatecall (a separate lowering); emitting
        // them here would collide with same-named contract functions.
        if let Some(Definition::Function(function)) = node.member().resolve_to_definition()
            && matches!(
                function.visibility(),
                FunctionVisibility::Internal
                    | FunctionVisibility::Private
                    | FunctionVisibility::Public
                    | FunctionVisibility::External
            )
            && function.compute_selector().is_none()
        {
            self.functions.push(function);
        }
        // Descend so nested calls (e.g. `L.f(L.g(x))`) are also recorded.
        true
    }
}

/// Returns the library functions reachable from `contract`'s own functions,
/// including those reached only through other library functions. The result is
/// deduplicated by node id and contains no contract-own functions.
pub(super) fn collect_library_functions(contract: &ContractDefinition) -> Vec<FunctionDefinition> {
    let own: HashSet<NodeId> = contract
        .compute_linearised_functions()
        .iter()
        .map(|function| function.node_id())
        .collect();
    let mut collected: HashMap<NodeId, FunctionDefinition> = HashMap::new();
    let mut walked: HashSet<NodeId> = HashSet::new();
    let mut to_walk: Vec<FunctionDefinition> = contract.compute_linearised_functions();

    while let Some(function) = to_walk.pop() {
        if !walked.insert(function.node_id()) {
            continue;
        }
        let mut collector = LibraryCallCollector::default();
        accept_function_definition(&function, &mut collector);
        for library_function in collector.functions {
            // Exclude the contract's own (inherited) functions reached via
            // `super.f`; those are emitted separately and re-registering them
            // would panic.
            if own.contains(&library_function.node_id()) {
                continue;
            }
            if collected
                .insert(library_function.node_id(), library_function.clone())
                .is_none()
            {
                // Newly seen — walk its body too, in case it reaches further
                // library functions.
                to_walk.push(library_function);
            }
        }
    }

    collected.into_values().collect()
}
