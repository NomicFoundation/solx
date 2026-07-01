//!
//! Calls whose callee is an identifier resolving to a function.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::PositionalArguments;

use solx_mlir::Function;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::call::CallContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::emit::emit_expression::EmitExpression;

impl<'emitter, 'state, 'context, 'block> CallContext<'emitter, 'state, 'context, 'block> {
    /// Emits a direct, named function call, returning all of its result values in declaration order.
    pub(super) fn emit_function_call(
        &self,
        function_definition: &FunctionDefinition,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let (mlir_name, argument_values, return_types, block) =
            self.emit_call_setup(function_definition, arguments, block);
        let results = Function::call(
            mlir_name,
            &argument_values,
            return_types,
            self.expression_context.state,
            &block,
        )
        .expect("function call resolves to a registered signature");
        BlockAnd {
            block,
            value: results,
        }
    }

    /// Emits argument values for a named call, resolves the callee's MLIR
    /// signature, and casts each argument to its declared parameter type.
    fn emit_call_setup<'a>(
        &'a self,
        function_definition: &FunctionDefinition,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> (
        &'a str,
        Vec<Value<'context, 'block>>,
        &'a [Type<'context>],
        BlockRef<'context, 'block>,
    ) {
        let mut argument_values = Vec::new();
        let mut current_block = block;
        for argument in arguments.iter() {
            let BlockAnd { value, block: next } = argument.emit(self.expression_context, current_block);
            argument_values.push(value);
            current_block = next;
        }

        let (mlir_name, parameter_types, return_types) = self
            .expression_context
            .state
            .resolve_function(function_definition.node_id())
            .expect("callee resolves to a registered signature");

        let context = self.expression_context.state;
        for (value, &param_type) in argument_values.iter_mut().zip(parameter_types) {
            let conversion = TypeConversion::from_target_type(param_type, context);
            *value = conversion.emit(*value, context, &current_block);
        }

        (mlir_name, argument_values, return_types, current_block)
    }
}
