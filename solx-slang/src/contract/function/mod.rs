//!
//! Function definition emission to Sol dialect MLIR.
//!

pub mod expression;
pub mod parameter;
pub mod statement;

use slang_solidity_v2::ast::ContractDefinition as SlangContractDefinition;
use slang_solidity_v2::ast::FunctionDefinition as SlangFunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::FunctionMutability;

use solx_mlir::Context as MlirContext;
use solx_mlir::Place;
use solx_mlir::StateMutability;
use solx_mlir::Type;
use solx_mlir::Value;

use crate::contract::ContractDefinition;
use crate::contract::function::parameter::Parameter;
use crate::contract::function::statement::block::Statements;
use crate::scope::ContractScope;

codegen!(
    FunctionDefinition {
        /// Emits `function`'s `sol.func` into the contract body from its pre-registered
        /// signature, binding parameters and named-return pointers into a fresh function frame.
        /// A constructor runs the contract's state variable initializers as its prologue.
        pub fn emit(
            function: &SlangFunctionDefinition,
            contract: &SlangContractDefinition,
            scope: &mut ContractScope,
        ) {
            let Some(body) = function.body() else {
                return;
            };
            let signature = scope.source_unit().function_signature(function.node_id());
            let entry = signature.define(
                function.compute_selector(),
                Self::state_mutability(function),
                Self::mlir_kind(function),
                scope,
                scope.contract_body(),
            );
            scope.function(entry, signature.return_types.clone(), |scope| {
                for (index, parameter) in function.parameters().iter().enumerate() {
                    let Some(identifier) = parameter.name() else {
                        continue;
                    };
                    scope.define_local(
                        identifier.name(),
                        signature.parameter_types[index],
                        |_context| entry.argument(index),
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
                                let return_type = signature.return_types[index];
                                if !return_type.is_integer() {
                                    unimplemented!(
                                        "zero-initialization for non-integer named return: {return_type}"
                                    );
                                }
                                Some(scope.define_local(identifier.name(), return_type, |scope| {
                                    Value::zero(return_type, scope)
                                }))
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                if matches!(function.kind(), FunctionKind::Constructor) {
                    ContractDefinition::emit_initializers(contract, scope);
                }

                Statements::emit(&body.statements(), scope);

                if !scope.current_block().is_terminated() {
                    let values: Vec<_> = signature
                        .return_types
                        .iter()
                        .zip(&return_pointers)
                        .map(|(&return_type, return_pointer)| match return_pointer {
                            Some(pointer) => pointer.load(return_type, scope),
                            None => Value::zero(return_type, scope),
                        })
                        .collect();
                    scope.current_block().r#return(&values, scope);
                }
            });
        }

        /// Emits the contract's `constructor()` `sol.func`, synthesizing an empty one that still
        /// runs the state variable initializers when the source declares no constructor.
        pub fn emit_constructor(contract: &SlangContractDefinition, scope: &mut ContractScope) {
            if let Some(constructor) = contract.constructor() {
                Self::emit(&constructor, contract, scope);
                return;
            }
            let entry = solx_mlir::Function::new("@constructor()".to_owned(), Vec::new(), Vec::new())
                .define(
                    None,
                    StateMutability::NonPayable,
                    Some(solx_mlir::FunctionKind::Constructor),
                    scope,
                    scope.contract_body(),
                );
            scope.function(entry, Vec::new(), |scope| {
                ContractDefinition::emit_initializers(contract, scope);
                scope.current_block().r#return(&[], scope);
            });
        }

        /// The declared parameter and return MLIR types, in declaration order.
        pub fn signature_types<'context>(
            function: &SlangFunctionDefinition,
            context: &MlirContext<'context>,
        ) -> (Vec<Type<'context>>, Vec<Type<'context>>) {
            let parameter_types = function
                .parameters()
                .iter()
                .map(|parameter| Parameter::resolve(&parameter, context))
                .collect();
            let return_types = function
                .returns()
                .map(|returns| {
                    returns
                        .iter()
                        .map(|parameter| Parameter::resolve(&parameter, context))
                        .collect()
                })
                .unwrap_or_default();
            (parameter_types, return_types)
        }

        /// The Sol dialect mutability attribute.
        pub fn state_mutability(function: &SlangFunctionDefinition) -> StateMutability {
            match function.attributes().mutability() {
                FunctionMutability::Pure => StateMutability::Pure,
                FunctionMutability::View => StateMutability::View,
                FunctionMutability::Payable => StateMutability::Payable,
                FunctionMutability::NonPayable => StateMutability::NonPayable,
            }
        }

        /// The Sol dialect function kind attribute; `None` for a regular function.
        pub fn mlir_kind(function: &SlangFunctionDefinition) -> Option<solx_mlir::FunctionKind> {
            match function.kind() {
                FunctionKind::Constructor => Some(solx_mlir::FunctionKind::Constructor),
                FunctionKind::Fallback => Some(solx_mlir::FunctionKind::Fallback),
                FunctionKind::Receive => Some(solx_mlir::FunctionKind::Receive),
                FunctionKind::Regular => None,
                FunctionKind::Modifier => unreachable!("modifiers are filtered before emission"),
            }
        }
    }
);
