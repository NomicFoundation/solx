//!
//! Function definition emission to Sol dialect MLIR.
//!

pub mod expression;
pub mod statement;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;

use solx_mlir::Function;
use solx_mlir::FunctionDispatch;
use solx_mlir::FunctionKind as MlirFunctionKind;
use solx_mlir::FunctionType;
use solx_mlir::Place;
use solx_mlir::StateMutability;
use solx_mlir::Value;

use crate::scope::contract::ContractScope;
use crate::scope::source_unit::SourceUnitScope;

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// Defines `function`'s `sol.func` in the contract body at its first naming, binding
    /// parameters and named-return pointers into a fresh function frame. A constructor runs the
    /// contract's state variable initializers as its prologue. A function carries its dispatch
    /// selector only in its owner's module, so a copied body never becomes an entry point.
    pub fn function_definition(&mut self, function: &FunctionDefinition) {
        let Some(body) = function.body() else {
            return;
        };
        if !self.defined_functions.insert(function.node_id()) {
            return;
        }
        let selector = match function.enclosing_definition() {
            Some(enclosing) if enclosing.node_id() == self.object_id => function.compute_selector(),
            _ => None,
        };
        let signature = self.source_unit.function_signature(function);
        let dispatch = FunctionDispatch::from(function);
        let state_mutability = StateMutability::from(function.attributes().mutability());
        let entry = signature.define(
            selector,
            dispatch,
            state_mutability,
            self,
            self.contract.body,
        );
        let FunctionType {
            parameters,
            results,
        } = signature.function_type;
        self.function(entry, results, |scope| {
            for (index, parameter) in function.parameters().iter().enumerate() {
                let Some(identifier) = parameter.name() else {
                    continue;
                };
                scope.define_local(identifier.name(), parameters[index], |_scope| {
                    entry.block.argument(index)
                });
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

            if matches!(function.kind(), FunctionKind::Constructor) {
                scope.state_variable_initializers();
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
    }

    /// Emits the contract's `constructor()` `sol.func`, synthesizing an empty one that still runs
    /// the state variable initializers when the source declares no constructor.
    pub fn constructor(&mut self, contract: &ContractDefinition) {
        if let Some(constructor) = contract.constructor() {
            self.function_definition(&constructor);
            return;
        }
        let entry = Function::constructor().define(
            None,
            FunctionDispatch::Kind(MlirFunctionKind::Constructor),
            StateMutability::NonPayable,
            self,
            self.contract.body,
        );
        self.function(entry, Vec::new(), |scope| {
            scope.state_variable_initializers();
            scope.current_block().r#return(&[], scope);
        });
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
