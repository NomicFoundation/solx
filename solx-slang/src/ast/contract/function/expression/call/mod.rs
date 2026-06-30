//!
//! Function call and member access expression emission.
//!

pub mod call_arguments;
pub mod contract_creation;
pub mod external_library_call;
pub mod external_member_call;
pub mod function_pointer_call;
pub mod identifier_builtin_call;
pub mod identifier_function_call;
pub mod index_access_conversion;
pub mod inherited_function_call;
pub mod internal_member_call;
pub mod member_builtin_call;
pub mod new_expression_call;
pub mod positional_arguments;
pub mod struct_construction;
pub mod try_external_call;
pub mod try_function_pointer_call;
pub mod try_new_expression;
pub mod type_conversion;

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::external_library_call::ExternalLibraryCall;
use crate::ast::contract::function::expression::call::external_member_call::ExternalMemberCall;
use crate::ast::contract::function::expression::call::function_pointer_call::FunctionPointerCall;
use crate::ast::contract::function::expression::call::identifier_builtin_call::IdentifierBuiltinCall;
use crate::ast::contract::function::expression::call::identifier_function_call::IdentifierFunctionCall;
use crate::ast::contract::function::expression::call::index_access_conversion::IndexAccessConversion;
use crate::ast::contract::function::expression::call::inherited_function_call::InheritedFunctionCall;
use crate::ast::contract::function::expression::call::internal_member_call::InternalMemberCall;
use crate::ast::contract::function::expression::call::member_builtin_call::MemberBuiltinCall;
use crate::ast::contract::function::expression::call::new_expression_call::NewExpressionCall;
use crate::ast::contract::function::expression::call::struct_construction::StructConstruction;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::call_options::CallOptions;

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

        if let Some(struct_construction) = StructConstruction::from_call(self, &callee) {
            return struct_construction.emit(context, block);
        }

        if let Some(type_conversion) = TypeConversion::from_call(self) {
            return type_conversion.emit(context, block);
        }

        if let Some(function_pointer_call) = FunctionPointerCall::from_callee(&callee, &arguments) {
            return function_pointer_call.emit(context, block, call_value, call_gas);
        }

        if let Some(identifier_builtin_call) =
            IdentifierBuiltinCall::from_callee(&callee, &arguments)
        {
            return identifier_builtin_call.emit(context, block);
        }

        if let Some(member_builtin_call) = MemberBuiltinCall::from_call(self, &callee) {
            return member_builtin_call.emit(context, block, call_value, call_gas);
        }

        if let Some(inherited_function_call) =
            InheritedFunctionCall::from_callee(&callee, &arguments, context.dispatch)
        {
            return inherited_function_call.emit(context, block);
        }

        if let Some(external_library_call) = ExternalLibraryCall::from_callee(&callee, &arguments) {
            return external_library_call.emit(context, block);
        }

        if let Some(internal_member_call) = InternalMemberCall::from_callee(&callee, &arguments) {
            return internal_member_call.emit(context, block);
        }

        if let Some(external_member_call) = ExternalMemberCall::from_callee(&callee, &arguments) {
            return external_member_call.emit(context, block, call_value, call_gas);
        }

        if let Some(new_expression_call) = NewExpressionCall::from_call(self, &callee) {
            return new_expression_call.emit(context, block, call_value, salt);
        }

        if let Some(index_access_conversion) = IndexAccessConversion::from_call(self, &callee) {
            return index_access_conversion.emit(context, block);
        }

        if let Some(identifier_function_call) =
            IdentifierFunctionCall::from_callee(&callee, &arguments, context.dispatch)
        {
            return identifier_function_call.emit(context, block);
        }

        if let Expression::Identifier(identifier) = &callee {
            unreachable!(
                "callee '{}' does not resolve to a function",
                identifier.name()
            );
        }
        unreachable!("unsupported callee expression");
    }
}
