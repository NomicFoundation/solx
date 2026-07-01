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
use solx_mlir::LocationPolicy;
use solx_mlir::Modifier;
use solx_mlir::Type as AstType;
use solx_mlir::ods::sol::ModifierCallBlkOperation;
use solx_mlir::ods::sol::ReturnOperation;

use crate::ast::analysis::query::positional_arguments::PositionalArguments;
use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::function::function_scope::FunctionScope;
use crate::ast::contract::function::mlir_symbol_name::MlirSymbolName;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_modifier_calls::EmitModifierCalls;
use crate::ast::emit::emit_statement::EmitStatement;

impl EmitModifierCalls for FunctionDefinition {
    fn resolve_invoked_modifiers<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
    ) -> Vec<FunctionDefinition> {
        self.modifier_invocations()
            .iter()
            .filter_map(|invocation| scope.resolve_modifier_invocation(&invocation))
            .collect()
    }

    fn emit_modifier_call_blocks<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        parameters: &[Parameter],
        parameter_types: &[Type<'context>],
        function_block: &BlockRef<'context, 'block>,
    ) {
        let state = scope.state;
        let block_arg_types: Vec<(Type<'context>, _)> = parameter_types
            .iter()
            .map(|parameter_type| (*parameter_type, state.location()))
            .collect();

        for invocation in self.modifier_invocations().iter() {
            let Some(definition) = scope.resolve_modifier_invocation(&invocation) else {
                continue;
            };

            // The `sol.modifier_call_blk` region has one `IsolatedFromAbove`, terminator-free block
            // whose arguments are a fresh copy of the whole wrapped-function parameter list; invocation
            // arguments must reference these block arguments, not `f`'s entry-block arguments.
            let region = Region::new();
            let block = Block::new(&block_arg_types);
            region.append_block(block);
            let block = region
                .first_block()
                .expect("the modifier-call block was just appended");

            let mut environment = Environment::new();
            for (index, parameter) in parameters.iter().enumerate() {
                let argument: Value<'context, '_> = block
                    .argument(index)
                    .expect("block argument index is within the signature")
                    .into();
                environment.bind_value(parameter.node_id(), argument);
            }

            let argument_expressions = invocation
                .arguments()
                .and_then(|argument_list| argument_list.positional_arguments())
                .unwrap_or_default();
            let mut current_block = block;
            let mut operands: Vec<Value<'context, '_>> = Vec::new();
            let mut parameter_types: Vec<Type<'context>> = Vec::new();
            for (parameter, argument) in definition.parameters().iter().zip(argument_expressions) {
                let BlockAnd {
                    value,
                    block: next_block,
                } = {
                    let emitter = ExpressionContext::new(
                        scope.state,
                        &environment,
                        scope.dispatch,
                        scope.storage_layout,
                        ArithmeticMode::Checked,
                    );
                    argument.emit(&emitter, current_block)
                };
                current_block = next_block;
                let parameter_type = AstType::resolve_optional(parameter.get_type(), state)
                    .expect("slang validated");
                let cast = value.cast(AstType::new(parameter_type), state, &current_block);
                operands.push(cast.into_mlir());
                parameter_types.push(parameter_type);
            }

            Modifier::new(definition.modifier_symbol(), parameter_types).call(
                &operands,
                state,
                &current_block,
            );

            function_block.append_operation(mlir_op_build!(
                state,
                ModifierCallBlkOperation.body_region(region)
            ));
        }
    }

    fn emit_modifier_definition<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        contract_body: &BlockRef<'context, '_>,
    ) {
        let Some(body) = self.body() else {
            return;
        };
        let state = scope.state;

        let parameter_types: Vec<Type<'context>> = self
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

        let definition = Modifier::new(self.modifier_symbol(), parameter_types.clone());
        let entry_block = definition.define(state, contract_body);

        let mut environment = Environment::new();
        for (index, parameter) in self.parameters().iter().enumerate() {
            environment.bind_block_argument(
                parameter.node_id(),
                parameter_types[index],
                index,
                &entry_block,
                state,
            );
        }

        let return_types: [Type<'context>; 0] = [];
        let mut current_block = entry_block;
        let mut terminated = false;
        for statement in body.statements().iter() {
            let mut emitter = StatementContext::new(
                scope.state,
                &mut environment,
                scope.dispatch,
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

        if !terminated {
            current_block.append_operation(mlir_op_build!(state, ReturnOperation.operands(&[])));
        }
    }
}
