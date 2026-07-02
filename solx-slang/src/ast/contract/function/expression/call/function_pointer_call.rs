//!
//! Calls whose callee is a value of internal function-pointer type.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Value as AstValue;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::call::CallContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::emit::emit_as::EmitAs;
use crate::ast::emit::emit_expression::EmitExpression;

impl<'emitter, 'state, 'context, 'block> CallContext<'emitter, 'state, 'context, 'block> {
    /// Emits a call through a function-pointer value `fp(a, b)` / `s.f(a, b)`, returning all of its
    /// result values in declaration order. The callee emits to a `!sol.func_ref<...>` value, whose
    /// `sol.icall` dispatches to the pointed-to function.
    pub(super) fn emit_function_pointer_call(
        &self,
        callee: &Expression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let context = self.expression_context;
        let function_slang_type = callee.get_type().expect("slang types every function-pointer call");
        let SlangType::Function(_) = function_slang_type else {
            unreachable!("a function-pointer callee is always function-typed");
        };
        let (parameter_types, result_types) =
            TypeConversion::function_pointer_signature(&function_slang_type, context.state);

        let BlockAnd {
            value: callee_value,
            block,
        } = callee.emit(context, block);

        let mut argument_values = Vec::with_capacity(parameter_types.len());
        let mut block = block;
        for (argument, &parameter_type) in arguments.iter().zip(&parameter_types) {
            let BlockAnd { value, block: next } = argument.emit_as(parameter_type, context, block);
            argument_values.push(value);
            block = next;
        }

        let results = AstValue::new(callee_value).call_indirect(
            &argument_values,
            &result_types,
            None,
            None,
            false,
            context.state,
            &block,
        );
        BlockAnd {
            block,
            value: results,
        }
    }
}
