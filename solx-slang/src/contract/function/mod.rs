//!
//! Function definition emission to Sol dialect MLIR.
//!

pub mod assembly;
pub mod expression;
pub mod statement;

use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;

use solx_mlir::Function;
use solx_mlir::FunctionDispatch;
use solx_mlir::FunctionKind as MlirFunctionKind;
use solx_mlir::Place;
use solx_mlir::StateMutability;
use solx_mlir::Value;

use crate::contract::object::Object;
use crate::scope::contract::ContractScope;
use crate::scope::source_unit::SourceUnitScope;

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// Defines `function`'s `sol.func` in the contract body at its first naming, binding
    /// parameters and named-return pointers into a fresh function frame. A function carries a
    /// dispatch selector only when the object dispatches it, so a copied or overridden body never
    /// becomes an entry point; its identifier is its node id, which Slang numbers from one, leaving
    /// the dialect's zero to the null pointer. The object's own constructor runs the hierarchy's
    /// state variable initializers as its prologue; every constructor then calls the next one in
    /// the chain, receiving the values threaded through it as trailing parameters, and a base
    /// constructor is reached by that call alone.
    pub fn function_definition(&mut self, function: &FunctionDefinition) {
        if !self.defined_functions.insert(function.node_id()) {
            return;
        }
        let body = function
            .body()
            .expect("slang admits a call naming a function declaration nothing implements");
        let position = self.chain.position(function);
        let dispatch = match function.kind() {
            FunctionKind::Constructor if position == Some(0) => {
                FunctionDispatch::Kind(MlirFunctionKind::Constructor)
            }
            FunctionKind::Constructor => FunctionDispatch::Symbol,
            FunctionKind::Regular => {
                FunctionDispatch::Identifier(usize::from(function.node_id()) as u64)
            }
            FunctionKind::Fallback => FunctionDispatch::Kind(MlirFunctionKind::Fallback),
            FunctionKind::Receive => FunctionDispatch::Kind(MlirFunctionKind::Receive),
            FunctionKind::Modifier => unreachable!("slang yields no modifier as a function"),
        };
        let selector = self
            .functions
            .iter()
            .any(|dispatched| dispatched.node_id() == function.node_id())
            .then(|| function.compute_selector())
            .flatten();
        let signature = self.signature(function);
        let entry = signature.define(
            selector,
            dispatch,
            StateMutability::from(function.attributes().mutability()),
            self,
            self.contract.body,
        );
        self.function(
            entry,
            position.is_some(),
            signature.function_type.results,
            |scope| {
                let parameters = function.parameters();
                let arguments: Vec<Value> = (0..parameters.len())
                    .map(|index| entry.argument(index))
                    .collect();
                scope.bind_parameters(&parameters, &arguments);

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

                if let Some(position) = position {
                    if position == 0 {
                        scope.state_variable_initializers();
                    }
                    scope.base_constructor_call(position, entry);
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
                                pointer
                                    .store(Value::default_initialized(return_type, scope), scope);
                                pointer.load(return_type, scope)
                            }
                        })
                        .collect();
                    scope.current_block().r#return(&values, scope);
                }
            },
        );
    }

    /// Emits the object's own constructor: the declared one, or a synthesized `constructor()` that
    /// still runs the state variable initializers and the base constructors.
    pub fn constructor(&mut self) {
        let Object::Contract(_) = &self.object else {
            return;
        };
        if let Some(constructor) = self.chain.constructor(0).cloned() {
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
        self.function(entry, true, Vec::new(), |scope| {
            scope.state_variable_initializers();
            scope.base_constructor_call(0, entry);
            scope.current_block().r#return(&[], scope);
        });
    }

    /// The symbol and MLIR signature `function` is defined and called by: a base constructor's
    /// carries the parameters threaded through it, which the chain lays out; any other function's
    /// is the source unit's.
    pub fn signature(&mut self, function: &FunctionDefinition) -> Function<'context> {
        match self.chain.position(function) {
            Some(position) if position > 0 => self.chain.signature(position, self.source_unit),
            _ => self.source_unit.function_signature(function),
        }
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
