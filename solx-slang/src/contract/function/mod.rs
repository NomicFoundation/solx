//!
//! Function definition emission to Sol dialect MLIR.
//!

pub mod expression;
pub mod statement;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::FunctionMutability;

use solx_mlir::Function;
use solx_mlir::Place;
use solx_mlir::StateMutability;
use solx_mlir::Value;

use crate::scope::contract::ContractScope;

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// Emits `function`'s `sol.func` into the contract body from its pre-registered signature,
    /// binding parameters and named-return pointers into a fresh function frame. A constructor runs
    /// the contract's state variable initializers as its prologue.
    pub fn function_definition(&mut self, function: &FunctionDefinition) {
        let Some(body) = function.body() else {
            return;
        };
        let signature = self.source_unit.function_signature(function.node_id());
        let state_mutability = match function.attributes().mutability() {
            FunctionMutability::Pure => StateMutability::Pure,
            FunctionMutability::View => StateMutability::View,
            FunctionMutability::Payable => StateMutability::Payable,
            FunctionMutability::NonPayable => StateMutability::NonPayable,
        };
        let mlir_kind = match function.kind() {
            FunctionKind::Constructor => Some(solx_mlir::FunctionKind::Constructor),
            FunctionKind::Fallback => Some(solx_mlir::FunctionKind::Fallback),
            FunctionKind::Receive => Some(solx_mlir::FunctionKind::Receive),
            FunctionKind::Regular => None,
            FunctionKind::Modifier => unreachable!("modifiers are filtered before emission"),
        };
        let entry = signature.define(
            function.compute_selector(),
            state_mutability,
            mlir_kind,
            self,
            self.contract_body,
        );
        let Function {
            parameter_types,
            return_types,
            ..
        } = signature;
        self.function(entry, return_types, |scope| {
            for (index, parameter) in function.parameters().iter().enumerate() {
                let Some(identifier) = parameter.name() else {
                    continue;
                };
                scope.define_local(identifier.name(), parameter_types[index], |_scope| {
                    entry.argument(index)
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
            StateMutability::NonPayable,
            Some(solx_mlir::FunctionKind::Constructor),
            self,
            self.contract_body,
        );
        self.function(entry, Vec::new(), |scope| {
            scope.state_variable_initializers();
            scope.current_block().r#return(&[], scope);
        });
    }
}
