//!
//! Identifier references: constants fold, variables load from their places.
//!

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Identifier;

use solx_mlir::Place;
use solx_mlir::Type;
use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// A constant folds to its initializer; every other identifier loads from its place.
    pub fn identifier(&mut self, node: &Identifier) -> Value<'context> {
        if let Some(Definition::Constant(constant)) = node.resolve_to_definition() {
            return self.expression(&constant.value().expect("constant has an initializer"));
        }
        let (place, element_type) = self.identifier_place(node);
        place.load(element_type, self)
    }

    /// A state variable resolves to its storage slot, a local variable or parameter to its stack
    /// pointer.
    pub fn identifier_place(&mut self, node: &Identifier) -> (Place<'context>, Type<'context>) {
        match node.resolve_to_definition() {
            Some(Definition::StateVariable(state_variable)) => {
                let slot_name = self
                    .contract
                    .storage_layout
                    .get(&state_variable.node_id())
                    .expect("state variable is registered in the storage layout")
                    .name
                    .clone();
                self.state_variable_place(&state_variable, &slot_name)
            }
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                self.environment.variable_with_type(&node.name())
            }
            None => unreachable!("slang resolves every identifier reference: {}", node.name()),
            Some(_) => unreachable!("identifier {} is not an assignable place", node.name()),
        }
    }
}
