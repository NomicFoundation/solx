//!
//! The external function-pointer call `try functionPointer(args)` a `try` statement guards.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::CallOptionsExpression;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Value as AstValue;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_kind::CallKind;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::call_options::CallOptions;
use crate::ast::emit::emit_as::EmitAs;
use crate::ast::emit::emit_expression::EmitExpression;

/// The external function-pointer call `try functionPointer(args)` guards, resolved ahead of emission
/// so [`Self::emit`] surfaces the call's success status for the `try` op's regions.
pub struct TryFunctionPointerCall {
    /// The `{value: v}` / `{gas: g}` options layer, if any (`functionPointer{value: v}(args)`).
    options: Option<CallOptionsExpression>,
    /// The external function-typed callee expression.
    callee: Expression,
    /// The call's positional arguments, coerced to the pointer's parameter types at emission.
    arguments: PositionalArguments,
}

impl TryFunctionPointerCall {
    /// Classifies `try functionPointer(args)`: a call through a value of external function-pointer
    /// type — a local, parameter, contract-static state variable, or struct field of function type,
    /// rather than a directly named external method. A direct external member call `try c.foo(args)`,
    /// an internal function-pointer call, or any other guarded shape yields `None`.
    pub fn from_expression(expression: &Expression) -> Option<Self> {
        let Expression::FunctionCallExpression(call) = expression else {
            return None;
        };
        let (options, callee) = match call.operand().unwrap_parentheses() {
            Expression::CallOptionsExpression(options) => {
                let callee = options.operand().unwrap_parentheses();
                (Some(options), callee)
            }
            callee => (None, callee),
        };
        if !CallKind::is_function_pointer_callee(&callee) {
            return None;
        }
        let Some(SlangType::Function(function_type)) = callee.get_type() else {
            return None;
        };
        if !function_type.is_externally_visible() {
            return None;
        }
        let ArgumentsDeclaration::PositionalArguments(arguments) = call.arguments() else {
            unreachable!("an external function-pointer call takes positional arguments");
        };
        Some(Self {
            options,
            callee,
            arguments,
        })
    }

    /// Emits the guarded call with `try` semantics, returning its success status, its result values in
    /// declaration order, and the continuation block.
    pub fn emit<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> (
        Value<'context, 'block>,
        Vec<Value<'context, 'block>>,
        BlockRef<'context, 'block>,
    ) {
        let function_slang_type = self.callee.get_type().expect("slang types every callee");
        let (parameter_types, result_types) =
            TypeConversion::function_pointer_signature(&function_slang_type, context.state);

        let BlockAnd {
            value: callee_value,
            block,
        } = self.callee.emit(context, block);

        let mut argument_values = Vec::with_capacity(parameter_types.len());
        let mut block = block;
        for (argument, &parameter_type) in self.arguments.iter().zip(&parameter_types) {
            let BlockAnd { value, block: next } = argument.emit_as(parameter_type, context, block);
            argument_values.push(value);
            block = next;
        }

        let (call_value, call_gas) = match &self.options {
            Some(options) => {
                let (value, _salt, gas, next_block) =
                    CallOptions(options).capture(context, block);
                block = next_block;
                (value, gas)
            }
            None => (None, None),
        };
        let (status, results) = AstValue::new(callee_value).external_call_indirect(
            &argument_values,
            &result_types,
            call_value,
            call_gas,
            false,
            true,
            context.state,
            &block,
        );
        (status, results, block)
    }
}
