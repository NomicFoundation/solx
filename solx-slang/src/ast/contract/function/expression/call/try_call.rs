//!
//! The external contract-instance method call a `try` statement guards.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::Type as SlangType;

use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::CallContext;

/// The external contract-instance method call `try c.foo(args)` guards, resolved ahead of emission so
/// [`Self::emit`] surfaces the call's success status for the `try` op's regions.
pub struct TryCall {
    /// The member access that selected the callee.
    access: MemberAccessExpression,
    /// The resolved external method carrying the ABI selector.
    function_definition: FunctionDefinition,
    /// The call's argument list, ordered against the callee's parameters at emission.
    arguments: ArgumentsDeclaration,
}

impl TryCall {
    /// Classifies `try c.foo(args)`: an external call to a contract- or interface-instance method
    /// carrying an ABI selector. Any other guarded shape yields `None`.
    pub fn from_expression(expression: &Expression) -> Option<Self> {
        let Expression::FunctionCallExpression(call) = expression else {
            return None;
        };
        let Expression::MemberAccessExpression(access) = call.operand().unwrap_parentheses() else {
            return None;
        };
        let Some(Definition::Function(function_definition)) =
            access.member().resolve_to_definition()
        else {
            return None;
        };
        if function_definition.compute_selector().is_none()
            || !matches!(
                access.operand().get_type(),
                Some(SlangType::Contract(_) | SlangType::Interface(_))
            )
        {
            return None;
        }
        Some(Self {
            access,
            function_definition,
            arguments: call.arguments(),
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
        CallContext::new(context).emit_external_member_call_fallible(
            &self.access,
            &self.function_definition,
            &self.arguments,
            None,
            None,
            true,
            block,
        )
    }
}
