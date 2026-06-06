//!
//! Collection of library functions inlined into a contract.
//!
//! Internal (no-selector) library functions are inlined into the calling
//! contract's MLIR module under the library's linker symbol. This pass walks a
//! contract's functions (transitively through the library and free functions
//! they reach) and returns every such library function, so the contract emitter
//! can pre-register and emit them.
//!

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionVisibility;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::visitor::Visitor;

/// Visitor that records library functions reached from a walked function body.
///
/// The SOLE top-level type of this module (§2a); the reachability walk is an
/// associated function on it (Rule-5).
#[derive(Default)]
pub struct LibraryCallCollector {
    /// Functions reached by member access (`L.f` / `x.f`).
    functions: Vec<FunctionDefinition>,
    /// Functions reached by bare identifier inside a library body.
    bare_functions: Vec<FunctionDefinition>,
    /// Every function reached by a bare-identifier reference.
    reached: Vec<FunctionDefinition>,
    /// Whether the function being walked is itself a library function.
    inside_library: bool,
}

impl LibraryCallCollector {
    /// Returns the library functions reachable from `contract`'s own functions,
    /// including those reached only through other library or free functions. The
    /// result is deduplicated by node id and contains no contract-own or free
    /// functions.
    ///
    /// `free_functions` is the source unit's full set of free functions (emitted
    /// separately, so excluded here). `extra_roots` are additional function
    /// bodies to walk that are not part of the linearised set.
    pub fn reachable_library_functions(
        contract: &ContractDefinition,
        free_functions: &[FunctionDefinition],
        extra_roots: &[FunctionDefinition],
    ) -> Vec<FunctionDefinition> {
        let _ = (contract, free_functions, extra_roots);
        unimplemented!("library-function reachability walk")
    }
}

impl Visitor for LibraryCallCollector {
    fn enter_expression(&mut self, node: &Expression) -> bool {
        // A bare-identifier reference inside a library body is a sibling library
        // function; a no-selector one is inlined here. Every bare reference is
        // also recorded so the caller can walk a reached free function's body.
        if let Expression::Identifier(identifier) = node
            && let Some(Definition::Function(function)) = identifier.resolve_to_definition()
        {
            if self.inside_library && function.compute_selector().is_none() {
                self.bare_functions.push(function.clone());
            }
            self.reached.push(function);
        }
        // Descend so nested references are also recorded.
        true
    }

    fn enter_member_access_expression(&mut self, node: &MemberAccessExpression) -> bool {
        // A member access whose member resolves to a no-selector library
        // function is an internal library call, inlined here — unless the operand
        // is a contract / `super` / `this`, in which case it is a contract
        // function dispatched through the super/base/virtual mechanism.
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
        // Descend so nested calls (e.g. `L.f(L.g(x))`) are also recorded.
        true
    }
}
