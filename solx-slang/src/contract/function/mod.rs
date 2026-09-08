//!
//! Function definition emission to Sol dialect MLIR.
//!

pub mod assembly;
pub mod expression;
pub mod statement;

use slang_solidity_v2::ast::FunctionDefinition;

use solx_mlir::Function;
use solx_mlir::FunctionDispatch;
use solx_mlir::FunctionKind as MlirFunctionKind;
use solx_mlir::Place;
use solx_mlir::StateMutability;
use solx_mlir::Value;

use crate::contract::constructor_chain::ConstructorChain;
use crate::contract::object::Object;
use crate::scope::contract::ContractScope;
use crate::scope::source_unit::SourceUnitScope;

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// Defines `function`'s `sol.func` in the contract body at its first naming, binding
    /// parameters and named-return pointers into a fresh function frame.
    pub fn function_definition(&mut self, function: &FunctionDefinition) -> Function<'context> {
        if !self.defined_members.insert(function.node_id()) {
            return self.source_unit.function_signature(function);
        }

        let position = self.chain.positions.get(&function.node_id()).copied();
        let body = function
            .body()
            .expect("slang admits a call naming a function declaration nothing implements");
        let selector = self
            .dispatched_functions
            .contains(&function.node_id())
            .then(|| function.compute_selector())
            .flatten();

        let mut signature = self.source_unit.function_signature(function);
        if let Some(position) = position {
            for parameter_position in self
                .chain
                .parameter_positions(position)
                .filter(|parameter_position| *parameter_position != position)
            {
                signature.function_type.parameters.extend(
                    self.source_unit
                        .function_signature(
                            self.chain.constructors[parameter_position]
                                .as_ref()
                                .expect("every position the chain threads holds a constructor"),
                        )
                        .function_type
                        .parameters,
                );
            }
        }

        let entry = signature.define(
            selector,
            FunctionDispatch::new(function, position == Some(ConstructorChain::MOST_DERIVED)),
            StateMutability::from(function.attributes().mutability()),
            self,
            self.contract.body,
        );

        self.function(entry, position.is_some(), &signature, |scope| {
            scope.bind_parameters(&function.parameters(), &entry.arguments());

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
                if position == ConstructorChain::MOST_DERIVED {
                    scope.state_variable_initializers();
                }
                scope.base_constructor_call(position, entry);
            }
            scope.modifier_invocations(function);

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

    /// Emits the object's own constructor: the declared one, or a synthesized `constructor()` that
    /// still runs the state variable initializers and the base constructors.
    pub fn constructor(&mut self) {
        let Object::Contract(_) = &self.object else {
            return;
        };
        if let Some(constructor) = self.chain.constructors[ConstructorChain::MOST_DERIVED].as_ref()
        {
            self.function_definition(constructor);
            return;
        }

        let entry = Function::constructor().define(
            None,
            FunctionDispatch::Kind(MlirFunctionKind::Constructor),
            StateMutability::NonPayable,
            self,
            self.contract.body,
        );

        self.function(entry, true, &Function::constructor(), |scope| {
            scope.state_variable_initializers();
            scope.base_constructor_call(ConstructorChain::MOST_DERIVED, entry);
            scope.current_block().r#return(&[], scope);
        });
    }
}

impl<'context> SourceUnitScope<'context> {
    /// The symbol of a function or modifier definition: its internal signature qualified by the
    /// node id, since internal signatures alone collide.
    pub fn function_symbol(function: &FunctionDefinition) -> String {
        format!(
            "{}_{}",
            function
                .compute_internal_signature()
                .expect("every emitted definition has an internal signature"),
            function.node_id(),
        )
    }
}
