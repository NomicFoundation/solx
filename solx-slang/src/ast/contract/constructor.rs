//!
//! Contract constructor synthesis: the deploy-time `constructor()` `sol.func` and the base-constructor
//! `sol.func`s the construction chain calls into.
//!

use std::collections::HashMap;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::ContractBase;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Environment;
use solx_mlir::Function;
use solx_mlir::StateMutability;
use solx_mlir::ods::sol::ReturnOperation;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::EmitStatement;
use crate::ast::LocationPolicy;
use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::analysis::query::BaseConstructorArguments;
use crate::ast::analysis::query::BaseConstructorChain;
use crate::ast::contract::function::FunctionScope;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::function::mlir_symbol_name::MlirSymbolName;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::emit::EmitConstructor;
use crate::ast::emit::EmitModifierCalls;

impl EmitConstructor for ContractDefinition {
    fn emit_constructor<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        contract_body: &BlockRef<'context, '_>,
    ) {
        let mro: Vec<ContractDefinition> = self
            .linearised_bases()
            .into_iter()
            .filter_map(|base| match base {
                ContractBase::Contract(base_contract) => Some(base_contract),
                ContractBase::Interface(_) => None,
            })
            .collect();
        let mro_node_ids = mro.iter().map(|base| base.node_id()).collect();

        let base_arguments = self.base_constructor_arguments(&mro, &mro_node_ids);

        self.emit_constructor_func(scope, self, &mro, &base_arguments, true, contract_body);

        for contract in mro.iter().skip(1) {
            if contract.constructor().is_none() {
                continue;
            }
            self.emit_constructor_func(
                scope,
                contract,
                &mro,
                &base_arguments,
                false,
                contract_body,
            );
        }
    }

    fn emit_constructor_func<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        owner: &ContractDefinition,
        mro: &[ContractDefinition],
        base_arguments: &HashMap<NodeId, BaseConstructorArguments>,
        is_most_derived: bool,
        contract_body: &BlockRef<'context, '_>,
    ) {
        let constructor = owner.constructor();
        let state = scope.state;

        let (symbol, kind, function_id) = if is_most_derived {
            (
                "constructor()".to_owned(),
                Some(solx_mlir::FunctionKind::Constructor),
                None,
            )
        } else {
            (
                constructor
                    .as_ref()
                    .expect(
                        "a base constructor func is emitted only for a contract with a constructor",
                    )
                    .base_constructor_symbol(),
                None,
                Some(scope.state.next_function_identifier()),
            )
        };

        let (parameter_types, mutability) = match &constructor {
            Some(constructor) => {
                let (parameter_types, _) =
                    AstType::resolve_signature(constructor, LocationPolicy::Declared(None), state);
                (
                    parameter_types,
                    StateMutability::from(constructor.mutability()),
                )
            }
            None => (Vec::new(), StateMutability::NonPayable),
        };

        let signature = Function::new(symbol, parameter_types, Vec::new());
        let entry = signature.define(None, mutability, kind, function_id, state, contract_body);
        let region = entry.parent_region().expect("entry block has a region");

        let mut current_block = if is_most_derived {
            self.emit_state_var_initializers(scope, entry)
        } else {
            entry
        };

        if let Some(constructor) = &constructor {
            let parameters: Vec<_> = constructor.parameters().iter().collect();
            constructor.emit_modifier_call_blocks(
                scope,
                &parameters,
                &signature.parameter_types,
                &current_block,
            );
        }

        let mut environment = Environment::new();
        if let Some(constructor) = &constructor {
            for (index, parameter) in constructor.parameters().iter().enumerate() {
                environment.bind_block_argument(
                    parameter.node_id(),
                    signature.parameter_types[index],
                    index,
                    &entry,
                    state,
                );
            }
        }

        current_block = self.emit_next_constructor_call(
            scope,
            owner,
            mro,
            base_arguments,
            &environment,
            current_block,
        );

        let mut terminated = false;
        if let Some(body) = constructor
            .as_ref()
            .and_then(|constructor| constructor.body())
        {
            let return_types: [Type<'context>; 0] = [];
            environment.enter_scope();
            for statement in body.statements().iter() {
                let mut emitter = StatementContext::new(
                    scope.state,
                    &mut environment,
                    scope.dispatch,
                    &region,
                    scope.storage_layout,
                    &return_types,
                    &[],
                );
                match statement.emit(&mut emitter, current_block) {
                    Some(next) => current_block = next,
                    None => {
                        terminated = true;
                        break;
                    }
                }
            }
            environment.exit_scope();
        }

        if !terminated {
            mlir_op_void!(state, &current_block, ReturnOperation.operands(&[]));
        }
    }

    fn emit_next_constructor_call<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        owner: &ContractDefinition,
        mro: &[ContractDefinition],
        base_arguments: &HashMap<NodeId, BaseConstructorArguments>,
        environment: &Environment<'context, 'block>,
        mut current_block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let Some(next_contract) = self.next_constructor_contract(owner, mro) else {
            return current_block;
        };
        let next_constructor = next_contract
            .constructor()
            .expect("next_constructor_contract returns a contract with a constructor");
        let state = scope.state;

        let arguments = base_arguments
            .get(&next_contract.node_id())
            .map(|spec| spec.arguments.as_slice())
            .unwrap_or_default();

        // Each argument keeps its OWN type (not the base constructor's parameter type); the
        // `sol.call` carries the implicit-castable operand/parameter mismatch, so no cast here.
        let mut operands: Vec<Value<'context, 'block>> = Vec::with_capacity(arguments.len());
        for argument in arguments.iter() {
            let emitter = ExpressionContext::new(
                scope.state,
                environment,
                scope.dispatch,
                scope.storage_layout,
                ArithmeticMode::Checked,
            );
            let BlockAnd {
                value,
                block: next_block,
            } = argument.emit(&emitter, current_block);
            current_block = next_block;
            operands.push(value.into_mlir());
        }

        let parameter_types: Vec<Type<'context>> = next_constructor
            .parameters()
            .iter()
            .map(|parameter| {
                AstType::resolve(
                    &parameter.get_type().expect("slang validated"),
                    LocationPolicy::Declared(None),
                    state,
                )
            })
            .collect();
        let next_signature = Function::new(
            next_constructor.base_constructor_symbol(),
            parameter_types,
            Vec::new(),
        );
        next_signature.call(&operands, state, &current_block);
        current_block
    }

    fn emit_state_var_initializers<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        mut block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let environment = Environment::new();
        let emitter = ExpressionContext::new(
            scope.state,
            &environment,
            scope.dispatch,
            scope.storage_layout,
            ArithmeticMode::Checked,
        );
        for state_variable in self.linearised_state_variables() {
            let Some(slot) = scope.storage_layout.get(&state_variable.node_id()) else {
                continue;
            };
            let Some(initializer) = state_variable.value() else {
                continue;
            };
            let declared_type = state_variable.get_type().expect("slang validated");
            let state = scope.state;
            let element_type =
                AstType::resolve(&declared_type, LocationPolicy::Declared(None), state);
            let address_type =
                AstType::new(element_type).address_type(slot.location, state.mlir_context);
            let storage_ref = Pointer::addr_of(&slot.name, address_type, state, &block).into_mlir();
            let BlockAnd {
                value,
                block: next_block,
            } = initializer.emit(&emitter, block);
            block = next_block;
            if declared_type.is_reference_type() {
                Pointer::new(storage_ref).copy_from(value, state, &block);
            } else {
                let stored_value = value.cast(AstType::new(element_type), state, &block);
                Pointer::new(storage_ref).store(stored_value, state, &block);
            }
        }
        block
    }
}
