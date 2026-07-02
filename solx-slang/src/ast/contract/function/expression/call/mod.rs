//!
//! Function call and member access expression emission.
//!

pub mod call_arguments;
pub mod call_kind;
pub mod contract_creation;
pub mod external_library_call;
pub mod external_member_call;
pub mod function_pointer_call;
pub mod identifier_builtin_call;
pub mod identifier_function_call;
pub mod inherited_function_call;
pub mod internal_member_call;
pub mod member_builtin_call;
pub mod new_expression_call;
pub mod positional_arguments;
pub mod struct_construction;
pub mod try_call;
pub mod try_call_kind;
pub mod try_new_expression;
pub mod type_conversion;

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_kind::CallKind;
use crate::ast::contract::function::expression::call_options::CallOptions;
use crate::ast::emit::emit_expression::EmitExpression;

impl<'context: 'block, 'block> EmitExpression<'context, 'block> for FunctionCallExpression {
    type Output = BlockAnd<'context, 'block, Vec<Value<'context, 'block>>>;

    /// Emits a function call, yielding its result values in declaration order: none for a void callee,
    /// one for a common callee, several for a tuple-returning call. The resolved callee selects the shape directly.
    fn emit<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> Self::Output {
        let (call_value, salt, call_gas, block, callee) = match self.operand().unwrap_parentheses()
        {
            Expression::CallOptionsExpression(options) => {
                let (value, salt, gas, block) = CallOptions(&options).capture(context, block);
                (
                    value,
                    salt,
                    gas,
                    block,
                    options.operand().unwrap_parentheses(),
                )
            }
            other => (None, None, None, block, other),
        };
        let arguments = self.arguments();

        match CallKind::from_call(self, &callee, &arguments, context.dispatch) {
            CallKind::StructConstruction(call) => call.emit(context, block),
            CallKind::TypeConversion(call) => call.emit(context, block),
            CallKind::FunctionPointerCall(call) => call.emit(context, block, call_value, call_gas),
            CallKind::IdentifierBuiltinCall(call) => call.emit(context, block),
            CallKind::MemberBuiltinCall(call) => call.emit(context, block, call_value, call_gas),
            CallKind::InheritedFunctionCall(call) => call.emit(context, block),
            CallKind::ExternalLibraryCall(call) => call.emit(context, block),
            CallKind::InternalMemberCall(call) => call.emit(context, block),
            CallKind::ExternalMemberCall(call) => call.emit(context, block, call_value, call_gas),
            CallKind::NewExpressionCall(call) => call.emit(context, block, call_value, salt),
            CallKind::IdentifierFunctionCall(call) => call.emit(context, block),
        }
    }
}
