//!
//! Function definition emission to Sol dialect MLIR.
//!

pub mod assembly;
pub mod expression;
pub mod statement;

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::VirtualTarget;

use solx_mlir::Function;
use solx_mlir::FunctionDispatch;
use solx_mlir::Place;
use solx_mlir::StateMutability;
use solx_mlir::Value;

use crate::contract::object::Object;
use crate::scope::contract::ContractScope;
use crate::scope::source_unit::SourceUnitScope;

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// Defines `function`'s `sol.func` in the contract body at its first naming, binding
    /// parameters and named-return pointers into a fresh function frame.
    pub fn function_definition(&mut self, function: &FunctionDefinition) -> Function<'context> {
        if !self.defined_functions.insert(function.node_id()) {
            return self.source_unit.function_signature(function);
        }

        let is_constructor = matches!(function.kind(), FunctionKind::Constructor);
        let is_most_derived_constructor = is_constructor
            && matches!(self.object, Object::Contract(contract) if contract
                .constructor()
                .is_some_and(|constructor| constructor.node_id() == function.node_id()));
        let body = function
            .body()
            .expect("slang admits a call naming a function declaration nothing implements");
        let selector = match (self.object, function.enclosing_definition()) {
            (Object::Contract(contract), Some(Definition::Contract(_)))
                if matches!(contract.resolve_virtual(function), VirtualTarget::Function(resolved)
                    if resolved.node_id() == function.node_id()) =>
            {
                function.compute_selector()
            }
            (Object::Library(library), Some(Definition::Library(enclosing)))
                if library.node_id() == enclosing.node_id() =>
            {
                function.compute_selector()
            }
            _ => None,
        };

        let mut signature = self.source_unit.function_signature(function);
        if is_constructor {
            signature
                .function_type
                .parameters
                .extend(self.constructor.parameter_types());
        }

        let entry = signature.define(
            selector,
            FunctionDispatch::new(function, is_most_derived_constructor),
            StateMutability::from(function.attributes().mutability()),
            self,
            self.contract.body,
        );

        if is_constructor {
            self.constructor.current = Some(function.node_id());
            self.constructor.bind_parameters(function, entry);
        }

        self.function(entry, is_constructor, &signature, |scope| {
            for (index, parameter) in function.parameters().iter().enumerate() {
                let Some(identifier) = parameter.name() else {
                    continue;
                };
                scope.define_local(
                    identifier.name(),
                    signature.function_type.parameters[index],
                    |_scope| entry.argument(index),
                );
            }

            let return_pointers: Vec<Option<Place>> = function
                .returns()
                .map(|returns| {
                    returns
                        .iter()
                        .enumerate()
                        .map(|(index, parameter)| {
                            let identifier = parameter.name()?;
                            let return_type = scope.return_types[index];
                            Some(scope.define_local(identifier.name(), return_type, |scope| {
                                Value::default_initialized(return_type, scope)
                            }))
                        })
                        .collect()
                })
                .unwrap_or_default();

            if is_most_derived_constructor {
                scope.state_variable_initializers();
            }
            if is_constructor {
                scope.base_constructor_call();
            }

            scope.statements(&body.statements());

            if !scope.current_block().is_terminated() {
                let values: Vec<_> = scope
                    .return_types
                    .iter()
                    .zip(&return_pointers)
                    .map(|(&return_type, return_pointer)| match return_pointer {
                        Some(pointer) => pointer.load(return_type, scope),
                        None => {
                            let pointer = Place::stack(return_type, scope);
                            pointer.store(Value::default_initialized(return_type, scope), scope);
                            pointer.load(return_type, scope)
                        }
                    })
                    .collect();
                scope.current_block().r#return(&values, scope);
            }
        });
        signature
    }
}

impl<'context> SourceUnitScope<'context> {
    /// The function's symbol: its internal signature qualified by the node id, since internal
    /// signatures alone collide.
    pub fn function_symbol(function: &FunctionDefinition) -> String {
        format!(
            "{}_{}",
            function
                .compute_internal_signature()
                .expect("every emitted function has an internal signature"),
            function.node_id(),
        )
    }
}
