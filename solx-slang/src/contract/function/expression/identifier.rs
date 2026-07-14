//!
//! Identifier references: constants fold, variables load from their places.
//!

use slang_solidity_v2::ast::Definition;

use crate::contract::function::expression::Expression;
use crate::contract::state_variable::StateVariableDefinition;

codegen!(
    Identifier {
        /// A constant folds to its initializer; everything else loads from its place.
        -> Value |node, scope| {
            if let Some(Definition::Constant(constant)) = node.resolve_to_definition() {
                let initializer = constant.value().expect("constant has an initializer");
                return Expression::emit(&initializer, scope);
            }
            let (place, element_type) = Self::emit_place(node, scope);
            place.load(element_type, scope)
        }

        /// A state variable resolves to its storage slot, a local variable or parameter to its
        /// stack pointer.
        -> Place |node, scope| {
            match node.resolve_to_definition() {
                Some(Definition::StateVariable(state_variable)) => {
                    let slot = scope
                        .contract()
                        .storage_layout()
                        .get(&state_variable.node_id())
                        .expect("state variable is registered in the storage layout");
                    StateVariableDefinition::storage_place(&state_variable, &slot.name, scope)
                }
                Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                    scope.environment().variable_with_type(&node.name())
                }
                None => unreachable!("slang resolves every identifier reference: {}", node.name()),
                Some(_) => unreachable!("identifier {} is not an assignable place", node.name()),
            }
        }
    }
);
