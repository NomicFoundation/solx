//!
//! The external contract-instance method call a `try` statement guards.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::CallOptionsExpression;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::StateVariableMutability;
use slang_solidity_v2::ast::Type as SlangType;

use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::CallContext;
use crate::ast::contract::function::expression::call_options::CallOptions;

/// The external contract-instance method or getter call `try c.foo(args)` guards, resolved ahead of
/// emission so [`Self::emit`] surfaces the call's success status for the `try` op's regions.
pub struct TryCall {
    /// The member access that selected the callee.
    access: MemberAccessExpression,
    /// The resolved external method or state-variable getter carrying the ABI selector.
    definition: Definition,
    /// The call's argument list, ordered against the callee's parameters at emission.
    arguments: ArgumentsDeclaration,
    /// The `{value: v}` / `{gas: g}` options layer, if any (`c.foo{value: v}(args)`).
    options: Option<CallOptionsExpression>,
}

impl TryCall {
    /// Classifies `try c.foo(args)`: an external call to a contract- or interface-instance method or
    /// `public` state-variable getter carrying an ABI selector. Any other guarded shape yields `None`.
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
        let Expression::MemberAccessExpression(access) = callee else {
            return None;
        };
        if !matches!(
            access.operand().get_type(),
            Some(SlangType::Contract(_) | SlangType::Interface(_))
        ) {
            return None;
        }
        let definition = match access.member().resolve_to_definition()? {
            Definition::Function(function_definition)
                if function_definition.compute_selector().is_some() =>
            {
                Definition::Function(function_definition)
            }
            Definition::StateVariable(state_variable)
                if state_variable.compute_selector().is_some()
                    && !matches!(
                        state_variable.mutability(),
                        StateVariableMutability::Constant | StateVariableMutability::Immutable
                    ) =>
            {
                Definition::StateVariable(state_variable)
            }
            _ => return None,
        };
        Some(Self {
            access,
            definition,
            arguments: call.arguments(),
            options,
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
        let mut block = block;
        let (call_value, call_gas) = match &self.options {
            Some(options) => {
                let (value, _salt, gas, next_block) =
                    CallOptions(options).capture(context, block);
                block = next_block;
                (value, gas)
            }
            None => (None, None),
        };
        CallContext::new(context).emit_external_member_call_fallible(
            &self.access,
            &self.definition,
            &self.arguments,
            call_value,
            call_gas,
            true,
            block,
        )
    }
}
