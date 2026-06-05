//!
//! Collection of internal library functions referenced by a contract.
//!
//! The Slang frontend compiles one contract per MLIR module, so a library's
//! internal (no-selector) functions are not emitted unless a contract reaches
//! one. This walks a contract's functions and constructor — transitively
//! through the library functions they reach — and returns every directly-called
//! internal library function (`L.f(...)` / `using`-for `x.f(...)`) so the
//! contract emitter can register and emit them into the contract body, where
//! they lower like ordinary internal functions. External / public library
//! functions (which carry a selector) are reached by delegatecall instead and
//! are excluded here.

use std::collections::HashMap;
use std::collections::HashSet;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionVisibility;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::visitor::Visitor;
use slang_solidity_v2::ast::visitor::accept_function_definition;

/// Visitor that records the internal library functions reached from a walked
/// function body: every `L.f` / `x.f` member access whose member resolves to a
/// no-selector library function, and — when walking *inside* a library body
/// (`inside_library`) — every bare-identifier sibling call `helper(...)` to
/// another library function in the same library (bare calls are unqualified
/// identifiers, not member accesses, so they would otherwise be missed).
#[derive(Default)]
struct LibraryCallCollector {
    /// Functions reached by member access (`L.f` / `x.f`); the caller filters
    /// out the contract's own functions.
    functions: Vec<FunctionDefinition>,
    /// Functions reached by bare identifier inside a library body. These may
    /// resolve to a free function, which the caller filters out (it is emitted
    /// separately under its own name) to avoid a double registration.
    bare_functions: Vec<FunctionDefinition>,
    /// Every function reached by a bare-identifier reference, regardless of
    /// context. The caller walks the *free* functions among these for the
    /// library calls they make.
    reached: Vec<FunctionDefinition>,
    /// Whether the function being walked is itself a library function, so a
    /// bare-identifier callee resolves to a sibling library function.
    inside_library: bool,
}

impl Visitor for LibraryCallCollector {
    fn enter_expression(&mut self, node: &Expression) -> bool {
        // A bare-identifier reference inside a library body is a sibling library
        // function. Only no-selector functions are inlined here; selector-bearing
        // ones are reached by delegatecall. Member-qualified references (`L.f`)
        // are handled by `enter_member_access_expression`.
        if let Expression::Identifier(identifier) = node
            && let Some(Definition::Function(function)) = identifier.resolve_to_definition()
        {
            if self.inside_library && function.compute_selector().is_none() {
                self.bare_functions.push(function.clone());
            }
            // Record every bare reference so the caller can walk a reached free
            // function's body for the library calls it makes.
            self.reached.push(function);
        }
        // Descend so nested references are also recorded.
        true
    }

    fn enter_member_access_expression(&mut self, node: &MemberAccessExpression) -> bool {
        // A member access whose member resolves to a no-selector library
        // function is an internal library call — `internal`/`private` functions,
        // but also `public` ones with non-ABI-encodable parameters, which solc
        // calls internally rather than by delegatecall. Selector-bearing library
        // functions are reached by delegatecall (a separate lowering).
        //
        // A `C.f(...)`, `super.f(...)` or `this.f(...)` member also resolves to a
        // no-selector function, but those are contract functions dispatched
        // through the base/super/virtual mechanism; collecting them would emit a
        // duplicate symbol into the contract body.
        let operand_is_contract_or_keyword = match node.operand() {
            Expression::Identifier(identifier) => matches!(
                identifier.resolve_to_definition(),
                Some(Definition::Contract(_))
            ),
            Expression::SuperKeyword(_) | Expression::ThisKeyword(_) => true,
            _ => false,
        };
        if !operand_is_contract_or_keyword
            && let Some(Definition::Function(function)) = node.member().resolve_to_definition()
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
        // Descend so nested calls (`L.f(L.g(x))`) are also recorded.
        true
    }
}

/// Returns the internal library functions reachable from `contract`'s own
/// functions and constructor, including those reached only through other
/// library functions (via member access `L.f` or, between siblings, by bare
/// name `helper(...)`). The result is deduplicated by node id and contains no
/// contract-own or free functions (those are registered and emitted elsewhere,
/// so including them would register the same node under two symbols).
pub fn collect_library_functions(
    contract: &ContractDefinition,
    free_functions: &[FunctionDefinition],
) -> Vec<FunctionDefinition> {
    let own: HashSet<NodeId> = contract
        .functions()
        .into_iter()
        .map(|function| function.node_id())
        .collect();
    let free_ids: HashSet<NodeId> = free_functions.iter().map(|free| free.node_id()).collect();

    let mut collected: HashMap<NodeId, FunctionDefinition> = HashMap::new();
    let mut walked: HashSet<NodeId> = HashSet::new();
    // The initial walk set (the contract's own functions + constructor) are
    // contract functions, so they collect only member-access (`L.f`) calls.
    let mut to_walk: Vec<FunctionDefinition> = contract.functions().into_iter().collect();
    if let Some(constructor) = contract.constructor() {
        to_walk.push(constructor);
    }
    // Library functions reached so far — walking one of these is "inside a
    // library", which enables bare-identifier sibling-call collection.
    let mut library_ids: HashSet<NodeId> = HashSet::new();

    while let Some(function) = to_walk.pop() {
        if !walked.insert(function.node_id()) {
            continue;
        }
        let mut collector = LibraryCallCollector {
            inside_library: library_ids.contains(&function.node_id()),
            ..LibraryCallCollector::default()
        };
        accept_function_definition(&function, &mut collector);
        // Member-access references are always library functions; bare-identifier
        // references may be free functions, which are excluded. Both exclude the
        // contract's own functions (reached via `C.f`).
        let member_reached = collector
            .functions
            .into_iter()
            .filter(|function| !own.contains(&function.node_id()));
        let bare_reached = collector.bare_functions.into_iter().filter(|function| {
            !own.contains(&function.node_id()) && !free_ids.contains(&function.node_id())
        });
        for library_function in member_reached.chain(bare_reached) {
            if collected
                .insert(library_function.node_id(), library_function.clone())
                .is_none()
            {
                // Newly seen — walk its body too, in case it reaches further
                // library functions; it is a library function, so its bare
                // sibling calls are collected when walked.
                library_ids.insert(library_function.node_id());
                to_walk.push(library_function);
            }
        }

        // Walk reached free functions for the library calls *they* make, without
        // collecting them (free functions are emitted separately under their own
        // name). This catches `function fu() { L.inter(); }` called as `fu()`.
        for reached_function in collector.reached {
            if free_ids.contains(&reached_function.node_id())
                && !walked.contains(&reached_function.node_id())
            {
                to_walk.push(reached_function);
            }
        }
    }

    collected.into_values().collect()
}
