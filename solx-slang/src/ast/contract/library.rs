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
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionVisibility;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::visitor::Visitor;
use slang_solidity_v2::ast::visitor::accept_function_definition;

/// Visitor that records library functions reached from a walked function body:
/// every `L.f` / `x.f` member access whose member resolves to a no-selector
/// library function, and — when walking *inside* a library body
/// (`inside_library`) — every bare-identifier sibling call `helper(...)` to
/// another library function in the same library. Sibling calls are unqualified
/// identifiers, not member accesses, so they would otherwise be missed.
#[derive(Default)]
struct LibraryCallCollector {
    /// Functions reached by member access (`L.f` / `x.f`), including
    /// `using`-attached free functions, which are registered under the library
    /// symbol — the caller filters only the contract's own functions.
    functions: Vec<FunctionDefinition>,
    /// Functions reached by bare identifier inside a library body. These may
    /// resolve to a free function, which the caller filters out (it is emitted
    /// separately under its plain name) to avoid a double registration.
    bare_functions: Vec<FunctionDefinition>,
    /// Whether the function being walked is itself a library function, so a
    /// bare-identifier callee resolves to a sibling library function.
    inside_library: bool,
}

impl Visitor for LibraryCallCollector {
    fn enter_expression(&mut self, node: &Expression) -> bool {
        // A bare-identifier reference inside a library body — whether called
        // (`helper(...)`) or taken as a function pointer (`p(helper)`) — is a
        // sibling library function. It cannot be a contract function (those are
        // unreachable by bare name from a library); a free function reached
        // this way is filtered out by the caller. Only no-selector functions
        // are inlined here; selector-bearing ones are reached by delegatecall.
        // Member-qualified references (`L.f`) are handled by
        // `enter_member_access_expression`.
        if self.inside_library
            && let Expression::Identifier(identifier) = node
            && let Some(Definition::Function(function)) = identifier.resolve_to_definition()
            && function.compute_selector().is_none()
        {
            self.bare_functions.push(function);
        }
        // Descend so nested references are also recorded.
        true
    }

    fn enter_member_access_expression(&mut self, node: &MemberAccessExpression) -> bool {
        // A member access whose member resolves to a library function with no
        // external selector is an internal library call — `internal`/`private`
        // functions, but also `public` ones with non-ABI-encodable parameters
        // (a `storage` / `mapping` argument), which solc calls internally
        // (passing the slot) rather than by `delegatecall`. Either way they are
        // inlined into the contract here. Library functions that *do* have a
        // selector are reached by delegatecall (a separate lowering); emitting
        // them here would collide with same-named contract functions.
        //
        // A direct `L.f(...)` (operand is a library/import qualifier) or a
        // `using for` `x.f(...)` (operand is a value) is a genuine library call.
        // A qualified base-contract call (`A.f(...)`), `super.f(...)` or
        // `this.f(...)` also resolves to a no-selector function, but those are
        // contract functions dispatched through the super/base/virtual
        // mechanism; collecting them would emit a duplicate function symbol into
        // the contract body (`redefinition of symbol`).
        let operand_is_contract_or_keyword = match node.operand() {
            Expression::Identifier(identifier) => {
                matches!(identifier.resolve_to_definition(), Some(Definition::Contract(_)))
            }
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
        // Descend so nested calls (e.g. `L.f(L.g(x))`) are also recorded.
        true
    }
}

/// Returns the library functions reachable from `contract`'s own functions,
/// including those reached only through other library functions (via member
/// access `L.f` or, between siblings, by bare name `helper(...)`). The result
/// is deduplicated by node id and contains no contract-own or free functions.
///
/// `free_functions` is the source unit's full set of free functions; they are
/// collected and emitted separately (under their plain name), so excluding
/// them here avoids registering the same node under two different symbols.
pub(super) fn collect_library_functions(
    contract: &ContractDefinition,
    free_functions: &[FunctionDefinition],
) -> Vec<FunctionDefinition> {
    let own: HashSet<NodeId> = contract
        .compute_linearised_functions()
        .iter()
        .map(|function| function.node_id())
        .collect();
    let free_ids: HashSet<NodeId> = free_functions.iter().map(|f| f.node_id()).collect();
    let mut collected: HashMap<NodeId, FunctionDefinition> = HashMap::new();
    let mut walked: HashSet<NodeId> = HashSet::new();
    let mut to_walk: Vec<FunctionDefinition> = contract.compute_linearised_functions();
    // Library functions reached so far — walking one of these is "inside a
    // library", which enables bare-identifier sibling-call collection. The
    // initial walk set (linearised functions + constructor) are contract
    // functions, so they collect only member-access (`L.f`) calls.
    let mut library_ids: HashSet<NodeId> = HashSet::new();
    // The constructor is not part of the linearised function set, but it can
    // call library functions too (`constructor() { L.f(); }`), so walk it.
    if let Some(constructor) = contract.constructor() {
        to_walk.push(constructor);
    }

    while let Some(function) = to_walk.pop() {
        if !walked.insert(function.node_id()) {
            continue;
        }
        let mut collector = LibraryCallCollector {
            inside_library: library_ids.contains(&function.node_id()),
            ..LibraryCallCollector::default()
        };
        accept_function_definition(&function, &mut collector);
        // Member-access references (`L.f` / `using`-attached `x.f`) are always
        // library functions; bare-identifier references may be free functions,
        // which are excluded (emitted separately under their plain name — a
        // second registration here would panic). Both exclude the contract's
        // own (inherited) functions reached via `super.f`.
        let member_reached = collector
            .functions
            .into_iter()
            .filter(|f| !own.contains(&f.node_id()));
        let bare_reached = collector
            .bare_functions
            .into_iter()
            .filter(|f| !own.contains(&f.node_id()) && !free_ids.contains(&f.node_id()));
        for library_function in member_reached.chain(bare_reached) {
            if collected
                .insert(library_function.node_id(), library_function.clone())
                .is_none()
            {
                // Newly seen — walk its body too, in case it reaches further
                // library functions. It is a library function, so its bare
                // sibling calls are collected when walked.
                library_ids.insert(library_function.node_id());
                to_walk.push(library_function);
            }
        }
    }

    collected.into_values().collect()
}
