//!
//! A function-pointer call in `try` position, classified ahead of emission.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::CallOptionsExpression;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::Type as SlangType;

use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::function_pointer_call::FunctionPointerCall;
use crate::ast::contract::function::expression::call_options::CallOptions;

/// A `try fp(args)` call through an externally-visible function pointer, optionally in a call-options
/// layer, resolved from the `try` expression so [`Self::emit`] is an infallible emitter.
pub struct TryFunctionPointerCall {
    /// The `{value: v}` / `{gas: g}` options layer, if any (`fp{value: v}(args)`).
    options: Option<CallOptionsExpression>,
    /// The classified function-pointer call.
    inner: FunctionPointerCall,
}

impl TryFunctionPointerCall {
    /// Classifies only when the `try` wraps a call through an externally-visible function pointer,
    /// optionally in a call-options layer; an internal pointer, invalid in `try`, or any other
    /// shape yields `None`.
    pub fn from_expression(expression: &Expression) -> Option<Self> {
        let Expression::FunctionCallExpression(call) = expression else {
            return None;
        };
        let (options, callee) = match call.operand() {
            Expression::CallOptionsExpression(options) => {
                let callee = options.operand();
                (Some(options), callee)
            }
            operand => (None, operand),
        };
        if !matches!(
            callee.get_type(),
            Some(SlangType::Function(function_type)) if function_type.is_externally_visible()
        ) {
            return None;
        }
        let inner = FunctionPointerCall::from_callee(&callee, &call.arguments())?;
        Some(Self { options, inner })
    }

    /// Emits this function-pointer call with `try` semantics, returning the success status flag, the
    /// decoded results, and the continuation block.
    pub fn emit<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> (
        Value<'context, 'block>,
        Vec<Value<'context, 'block>>,
        BlockRef<'context, 'block>,
    ) {
        let mut current_block = block;
        let mut call_value = None;
        let mut call_gas = None;
        if let Some(options) = &self.options {
            let (value, _salt, gas, next_block) =
                CallOptions(options).capture(context, current_block);
            current_block = next_block;
            call_value = value;
            call_gas = gas;
        }
        self.inner
            .emit_try(context, call_value, call_gas, current_block)
    }
}
