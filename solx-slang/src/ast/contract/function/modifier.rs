//!
//! Function- and constructor-modifier emission (`sol.modifier_call_blk` + `sol.modifier`).
//!
//! A modified function `f` evaluates each modifier invocation in its own `sol.modifier_call_blk`: an
//! `IsolatedFromAbove` block carrying a fresh copy of `f`'s whole parameter list as block arguments,
//! which evaluates the invocation's arguments and `sol.call`s the modifier. The blocks sit at the top
//! of `f`, before `f`'s inlined body. Each invoked modifier definition is emitted once as a contract-
//! level `sol.modifier`, with its `_;` emitted as `sol.placeholder`.
//!

use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::Parameter;

use solx_mlir::Environment;
use solx_mlir::Modifier;
use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::ModifierCallBlkOperation;
use solx_mlir::ods::sol::ReturnOperation;

use crate::ast::analysis::query::positional_arguments::PositionalArguments;
use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::FunctionEmitter;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_modifier_calls::EmitModifierCalls;
use crate::ast::emit::emit_statement::EmitStatement;

impl EmitModifierCalls for FunctionDefinition {
    fn resolve_invoked_modifiers<'state, 'context>(
        &self,
        emitter: &FunctionEmitter<'state, 'context>,
    ) -> Vec<FunctionDefinition> {
        self.modifier_invocations()
            .iter()
            .filter_map(|invocation| emitter.resolve_modifier_invocation(&invocation))
            .collect()
    }

    /// Emits a `sol.modifier_call_blk` region for each modifier invocation. Each region holds one
    /// `IsolatedFromAbove`, terminator-free block whose arguments are a fresh copy of the wrapped
    /// function's whole parameter list, so invocation arguments reference those block arguments
    /// rather than the wrapped function's entry-block arguments.
    fn emit_modifier_call_blocks<'state, 'context, 'block>(
        &self,
        emitter: &FunctionEmitter<'state, 'context>,
        parameters: &[Parameter],
        parameter_types: &[Type<'context>],
        function_block: &BlockRef<'context, 'block>,
    ) {
        let state = emitter.state();
        let block_arg_types: Vec<(Type<'context>, _)> = parameter_types
            .iter()
            .map(|parameter_type| (*parameter_type, state.location()))
            .collect();

        for invocation in self.modifier_invocations().iter() {
            let Some(definition) = emitter.resolve_modifier_invocation(&invocation) else {
                continue;
            };

            let region = Region::new();
            let block = Block::new(&block_arg_types);
            region.append_block(block);
            let block = region
                .first_block()
                .expect("the modifier-call block was just appended");

            let mut environment = Environment::new();
            for (index, parameter) in parameters.iter().enumerate() {
                let parameter_name = parameter
                    .name()
                    .map(|identifier| identifier.name())
                    .unwrap_or_else(|| "_".to_owned());
                let argument: Value<'context, '_> = block
                    .argument(index)
                    .expect("block argument index is within the signature")
                    .into();
                environment.bind_value(parameter_name, argument, parameter_types[index]);
            }

            let mut current_block = block;
            if let Some(returns) = self.returns() {
                for parameter in returns.iter() {
                    let Some(name) = parameter.name() else {
                        continue;
                    };
                    let slang_type = parameter.get_type().expect("slang types every return");
                    let return_type = TypeConversion::resolve_slang_type(&slang_type, None, state);
                    let pointer = Pointer::default_initialized(
                        &slang_type,
                        AstType::new(return_type),
                        state,
                        &current_block,
                    )
                    .into_mlir();
                    environment.define_variable(name.name(), pointer, return_type);
                }
            }

            let argument_expressions = invocation
                .arguments()
                .and_then(|argument_list| argument_list.positional_arguments())
                .unwrap_or_default();
            let (modifier_parameter_types, _) =
                TypeConversion::resolve_function_types(&definition, state);
            let mut operands: Vec<Value<'context, '_>> = Vec::new();
            for (parameter_type, argument) in
                modifier_parameter_types.iter().zip(argument_expressions)
            {
                let BlockAnd {
                    value,
                    block: next_block,
                } = {
                    let emitter = ExpressionContext::new(
                        state,
                        &environment,
                        emitter.storage_layout(),
                        emitter.dispatch(),
                        true,
                    );
                    argument.emit(&emitter, current_block)
                };
                current_block = next_block;
                let cast =
                    TypeConversion::from_target_type(*parameter_type, state).emit(value, state, &current_block);
                operands.push(cast);
            }

            Modifier::new(
                FunctionEmitter::modifier_symbol(&definition),
                modifier_parameter_types,
            )
            .call(&operands, state, &current_block);

            function_block.append_operation(mlir_op_build!(
                state,
                ModifierCallBlkOperation.body_region(region)
            ));
        }
    }

    fn emit_modifier_definition<'state, 'context>(
        &self,
        emitter: &FunctionEmitter<'state, 'context>,
        contract_body: &BlockRef<'context, '_>,
    ) {
        let Some(body) = self.body() else {
            return;
        };
        let state = emitter.state();

        let (parameter_types, _) = TypeConversion::resolve_function_types(self, state);

        let definition =
            Modifier::new(FunctionEmitter::modifier_symbol(self), parameter_types.clone());
        let entry_block = definition.define(state, contract_body);

        let mut environment = Environment::new();
        for (index, parameter) in self.parameters().iter().enumerate() {
            let parameter_name = parameter
                .name()
                .map(|identifier| identifier.name())
                .unwrap_or_else(|| "_".to_owned());
            let parameter_type = parameter_types[index];
            let parameter_value: Value<'context, '_> = entry_block
                .argument(index)
                .expect("modifier entry block has one argument per parameter")
                .into();
            let pointer = Pointer::stack(AstType::new(parameter_type), state, &entry_block);
            pointer.store(AstValue::new(parameter_value), state, &entry_block);
            environment.define_variable(parameter_name, pointer.into_mlir(), parameter_type);
        }

        let region = entry_block
            .parent_region()
            .expect("entry block belongs to a region");
        let return_types: [Type<'context>; 0] = [];
        let mut current_block = entry_block;
        let mut terminated = false;
        for statement in body.statements().iter() {
            let mut statement_context = StatementContext::new(
                state,
                &mut environment,
                &region,
                emitter.storage_layout(),
                emitter.dispatch(),
                &return_types,
            );
            match statement.emit(&mut statement_context, current_block) {
                Some(next) => current_block = next,
                None => {
                    terminated = true;
                    break;
                }
            }
        }

        if !terminated {
            mlir_op_void!(state, &current_block, ReturnOperation.operands(&[]));
        }
    }
}
