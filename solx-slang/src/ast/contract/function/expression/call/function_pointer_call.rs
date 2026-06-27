//!
//! Solidity function-pointer call.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::Type as SlangType;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::Type as AstType;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_arguments::CallArguments;

/// A call through a Solidity function-typed value.
pub struct FunctionPointerCall {
    /// The function-typed callee expression.
    pub callee: Expression,
    /// Positional arguments passed to the function pointer.
    pub arguments: CallArguments,
}

impl FunctionPointerCall {
    /// Classifies a callee as a function-pointer call.
    pub fn from_callee(callee: &Expression, arguments: &ArgumentsDeclaration) -> Option<Self> {
        let function_pointer_callee = match callee {
            Expression::Identifier(identifier) => matches!(
                identifier.resolve_to_definition(),
                Some(
                    Definition::Variable(_)
                        | Definition::Parameter(_)
                        | Definition::StateVariable(_)
                )
            ),
            Expression::MemberAccessExpression(access) => {
                match access.member().resolve_to_definition() {
                    Some(Definition::StructMember(_)) => true,
                    Some(Definition::StateVariable(_)) => matches!(
                        &access.operand(),
                        Expression::Identifier(operand)
                            if matches!(
                                operand.resolve_to_definition(),
                                Some(Definition::Contract(_))
                            )
                    ),
                    _ => false,
                }
            }
            _ => true,
        };
        if !function_pointer_callee || !matches!(callee.get_type(), Some(SlangType::Function(_))) {
            return None;
        }
        Some(Self {
            callee: callee.clone(),
            arguments: CallArguments::positional(arguments),
        })
    }

    /// Emits the function-pointer call.
    pub fn emit<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
        call_value: Option<Value<'context, 'block>>,
        call_gas: Option<Value<'context, 'block>>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let function_slang_type = self.callee.get_type().expect("slang validated");
        let (parameter_types, result_types) =
            AstType::function_pointer_signature(&function_slang_type, &context.state.builder);
        let BlockAnd {
            value: callee_value,
            block,
        } = self.callee.emit(context, block);
        let BlockAnd {
            value: argument_values,
            block,
        } = self.arguments.emit_as(&parameter_types, context, block);
        let results = callee_value.call_indirect(
            &argument_values,
            &result_types,
            call_value,
            call_gas,
            false,
            &context.state.builder,
            &block,
        );
        BlockAnd {
            value: results,
            block,
        }
    }
}
