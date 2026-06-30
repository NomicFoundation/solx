//!
//! An external call in `try` position, classified ahead of emission.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::CallOptionsExpression;
use slang_solidity_v2::ast::Expression;

use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::external_member_call::ExternalMemberCall;
use crate::ast::contract::function::expression::call_options::CallOptions;

/// A `try recv.f(args)` external member call, a function or a generated getter taking positional or
/// named arguments, resolved from the `try` expression, so [`Self::emit`] is an infallible emitter.
pub struct TryExternalCall {
    /// The `{value: v}` / `{gas: g}` options layer, if any (`recv.f{value: v}(args)`).
    options: Option<CallOptionsExpression>,
    /// The classified external member call.
    inner: ExternalMemberCall,
}

impl TryExternalCall {
    /// Classifies only when the `try` wraps an external member call `recv.f(args)`, optionally in a
    /// call-options layer; any other shape yields `None`.
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
        let inner = ExternalMemberCall::from_callee(&callee, &call.arguments())?;
        Some(Self { options, inner })
    }

    /// Emits this external call with `try` semantics, returning the success
    /// status flag, the decoded results, and the continuation block.
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
