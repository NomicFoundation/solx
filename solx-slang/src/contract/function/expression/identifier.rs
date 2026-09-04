//!
//! Identifier references: constants fold, function names materialise pointers, variables load from
//! their places.
//!

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::Identifier;
use slang_solidity_v2::ast::StateVariableMutability;
use slang_solidity_v2::ast::Type;

use solx_mlir::FunctionDispatch;
use solx_mlir::FunctionKind;
use solx_mlir::Place;
use solx_mlir::Type as MlirType;
use solx_mlir::Value;

use crate::contract::object::Object;
use crate::scope::function::FunctionScope;
use crate::scope::source_unit::SourceUnitScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// A constant folds to its initializer; an immutable outside the constructor loads its linked
    /// value (`sol.load_immutable`); a bare function name materialises its internal pointer
    /// (`sol.func_constant`), defining the function in this module if absent; a library name is its
    /// linked address (`sol.lib_addr`); every other identifier loads from its place.
    pub fn identifier(&mut self, node: &Identifier) -> Value<'context> {
        let definition = node.resolve_to_definition();
        if let Some(definition) = &definition
            && let Some((initializer, _)) = Self::constant_definition(definition)
        {
            return self.expression(&initializer);
        }
        match definition {
            Some(Definition::StateVariable(state_variable))
                if let StateVariableMutability::Immutable =
                    state_variable.attributes().mutability()
                    && self.dispatch != FunctionDispatch::Kind(FunctionKind::Constructor) =>
            {
                let element_type = self.resolve_type(
                    &state_variable
                        .get_type()
                        .expect("binder types every state variable"),
                    None,
                );
                Value::load_immutable(
                    &SourceUnitScope::state_variable_symbol(&state_variable),
                    element_type,
                    self,
                )
            }
            Some(Definition::Function(function)) => {
                self.contract.function_definition(&function);
                self.contract
                    .source_unit
                    .function_signature(&function)
                    .pointer_constant(self)
            }
            Some(Definition::Library(library)) => {
                Value::library_address(Object::Library(library).identifier().as_str(), self)
            }
            _ => {
                let (place, element_type) = self.identifier_place(node);
                place.load(element_type, self)
            }
        }
    }

    /// The initializer and declared type of a compile-time constant - a file-level `constant` or a
    /// `constant` state variable - absent for every other definition.
    pub fn constant_definition(definition: &Definition) -> Option<(Expression, Option<Type>)> {
        match definition {
            Definition::Constant(constant) => Some((
                constant.value().expect("a constant has an initializer"),
                constant.get_type(),
            )),
            Definition::StateVariable(state_variable)
                if let StateVariableMutability::Constant =
                    state_variable.attributes().mutability() =>
            {
                Some((
                    state_variable
                        .value()
                        .expect("a constant state variable is initialized"),
                    state_variable.get_type(),
                ))
            }
            _ => None,
        }
    }

    /// A state variable resolves to its storage slot, a local variable or parameter to its stack
    /// pointer.
    pub fn identifier_place(&mut self, node: &Identifier) -> (Place<'context>, MlirType<'context>) {
        match node.resolve_to_definition() {
            Some(Definition::StateVariable(state_variable)) => {
                self.state_variable_place(&state_variable)
            }
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                self.environment.variable_with_type(node.name())
            }
            None => unreachable!("slang resolves every identifier reference: {}", node.name()),
            Some(_) => unreachable!("identifier {} is not an assignable place", node.name()),
        }
    }
}
