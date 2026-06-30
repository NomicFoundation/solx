//!
//! Collection of library functions inlined into a contract.
//!
//! Internal (no-selector) library functions are inlined into the calling contract's module. This pass
//! returns every such function reached transitively from a contract's functions, for the emitter to register.
//!

use std::collections::HashSet;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::visitor::Visitor;
use slang_solidity_v2::ast::visitor::accept_function_definition;

use crate::ast::analysis::walk::body_origin::BodyOrigin;
use crate::ast::analysis::walk::reachability::ReachabilityWalk;

/// Visitor that records library functions reached from a walked function body.
#[derive(Default)]
pub struct LibraryCallCollector {
    /// Functions reached by member access (`L.f` / `x.f`).
    functions: Vec<FunctionDefinition>,
    /// Functions reached by bare identifier inside a library body.
    bare_functions: Vec<FunctionDefinition>,
    /// Every function reached by a bare-identifier reference.
    reached: Vec<FunctionDefinition>,
    /// Where the function being walked originates (a library body enables
    /// bare-identifier sibling-call collection).
    origin: BodyOrigin,
}

impl LibraryCallCollector {
    /// Returns the library functions reachable from `contract`'s own functions (deduplicated by node
    /// id, excluding contract-own and free functions). `extra_roots` are extra bodies to walk.
    pub fn reachable_library_functions(
        contract: &ContractDefinition,
        free_functions: &[FunctionDefinition],
        extra_roots: &[FunctionDefinition],
    ) -> Vec<FunctionDefinition> {
        let own: HashSet<NodeId> = contract
            .linearised_functions()
            .iter()
            .map(|function| function.node_id())
            .collect();
        let free_ids: HashSet<NodeId> = free_functions
            .iter()
            .map(|function| function.node_id())
            .collect();
        let mut walk = ReachabilityWalk::new(contract, extra_roots);
        let mut library_ids: HashSet<NodeId> = HashSet::new();

        while let Some(function) = walk.next_body() {
            let mut collector = Self {
                origin: if library_ids.contains(&function.node_id()) {
                    BodyOrigin::Library
                } else {
                    BodyOrigin::Contract
                },
                ..Self::default()
            };
            accept_function_definition(&function, &mut collector);
            let member_reached = collector
                .functions
                .into_iter()
                .filter(|function| !own.contains(&function.node_id()));
            let bare_reached = collector.bare_functions.into_iter().filter(|function| {
                !own.contains(&function.node_id()) && !free_ids.contains(&function.node_id())
            });
            for library_function in member_reached.chain(bare_reached) {
                if !walk.is_collected(library_function.node_id()) {
                    library_ids.insert(library_function.node_id());
                }
                walk.reach(library_function);
            }
            for reached_function in collector.reached {
                if free_ids.contains(&reached_function.node_id()) {
                    walk.enqueue(reached_function);
                }
            }
        }

        walk.into_reached()
    }
}

impl Visitor for LibraryCallCollector {
    fn enter_expression(&mut self, node: &Expression) -> bool {
        if let Expression::Identifier(identifier) = node
            && let Some(Definition::Function(function)) = identifier.resolve_to_definition()
        {
            if matches!(self.origin, BodyOrigin::Library) && function.compute_selector().is_none() {
                self.bare_functions.push(function.clone());
            }
            self.reached.push(function);
        }
        true
    }

    fn enter_member_access_expression(&mut self, node: &MemberAccessExpression) -> bool {
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
            && function.compute_selector().is_none()
        {
            self.functions.push(function);
        }
        true
    }
}
