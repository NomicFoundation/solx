//!
//! A `try`-guarded call: an optional `{value: v}` / `{gas: g}` call-options layer over an inner
//! infallible `try` emitter, classified ahead of emission. The inner call is either an external
//! member call or a call through an externally-visible function pointer.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::CallOptionsExpression;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::Type as SlangType;

use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::external_member_call::ExternalMemberCall;
use crate::ast::contract::function::expression::call::function_pointer_call::FunctionPointerCall;
use crate::ast::contract::function::expression::call_options::CallOptions;

/// The inner call a [`TryCall`] drives with `try` semantics.
pub trait EmitTry {
    /// Emits the inner call with `try` semantics, given the value and gas captured from the options
    /// layer, returning the success status flag, the decoded results, and the continuation block.
    fn emit_try<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        call_value: Option<Value<'context, 'block>>,
        call_gas: Option<Value<'context, 'block>>,
        block: BlockRef<'context, 'block>,
    ) -> (
        Value<'context, 'block>,
        Vec<Value<'context, 'block>>,
        BlockRef<'context, 'block>,
    );
}

/// A `try`-guarded call: an optional call-options layer over an inner infallible `try` emitter,
/// resolved from the `try` expression so [`Self::emit`] is total.
pub struct TryCall<T> {
    /// The `{value: v}` / `{gas: g}` options layer, if any.
    options: Option<CallOptionsExpression>,
    /// The classified inner call.
    inner: T,
}

impl<T> TryCall<T> {
    /// Splits a `try` call expression into its optional `{value}` / `{gas}` options layer, the
    /// callee, and the argument list; any non-call shape yields `None`.
    fn split_call(
        expression: &Expression,
    ) -> Option<(
        Option<CallOptionsExpression>,
        Expression,
        ArgumentsDeclaration,
    )> {
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
        Some((options, callee, call.arguments()))
    }
}

impl TryCall<ExternalMemberCall> {
    /// Classifies `try recv.f(args)`, optionally in a call-options layer; any other shape yields `None`.
    pub fn from_expression(expression: &Expression) -> Option<Self> {
        let (options, callee, arguments) = Self::split_call(expression)?;
        let inner = ExternalMemberCall::from_callee(&callee, &arguments)?;
        Some(Self { options, inner })
    }
}

impl TryCall<FunctionPointerCall> {
    /// Classifies `try fp(args)` through an externally-visible function pointer, optionally in a
    /// call-options layer; an internal pointer, invalid in `try`, or any other shape yields `None`.
    pub fn from_expression(expression: &Expression) -> Option<Self> {
        let (options, callee, arguments) = Self::split_call(expression)?;
        if !matches!(
            callee.get_type(),
            Some(SlangType::Function(function_type)) if function_type.is_externally_visible()
        ) {
            return None;
        }
        let inner = FunctionPointerCall::from_callee(&callee, &arguments)?;
        Some(Self { options, inner })
    }
}

impl<T: EmitTry> TryCall<T> {
    /// Captures the call-options layer, then emits the inner call with `try` semantics.
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
