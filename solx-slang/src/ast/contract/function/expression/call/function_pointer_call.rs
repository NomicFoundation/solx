//!
//! Solidity function-pointer call.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::ods::sol::ExtICallOperation;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
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
            AstType::function_pointer_signature(&function_slang_type, context.state);
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
            context.state,
            &block,
        );
        BlockAnd {
            value: results,
            block,
        }
    }

    /// Emits this call with `try` semantics, returning the success status flag, the decoded results,
    /// and the continuation block. Valid only for an externally-visible function pointer.
    pub fn emit_try<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        call_value: Option<Value<'context, 'block>>,
        call_gas: Option<Value<'context, 'block>>,
        block: BlockRef<'context, 'block>,
    ) -> (
        Value<'context, 'block>,
        Vec<Value<'context, 'block>>,
        BlockRef<'context, 'block>,
    ) {
        let function_slang_type = self.callee.get_type().expect("slang validated");
        let (parameter_types, result_types) =
            AstType::function_pointer_signature(&function_slang_type, context.state);
        let BlockAnd {
            value: callee_value,
            block,
        } = self.callee.emit(context, block);
        let BlockAnd {
            value: argument_values,
            block,
        } = self.arguments.emit_as(&parameter_types, context, block);
        let state = context.state;
        let value = call_value.unwrap_or_else(|| AstValue::uint256(0, state, &block).into_mlir());
        let gas = call_gas.unwrap_or_else(|| AstValue::gas_left(state, &block).into_mlir());
        let mut out_types = Vec::with_capacity(result_types.len() + 1);
        out_types.push(AstType::signless(state.mlir(), solx_utils::BIT_LENGTH_BOOLEAN).into_mlir());
        out_types.extend_from_slice(&result_types);
        let operation = block.append_operation(mlir_op_build!(
            state,
            ExtICallOperation
                .outs(&out_types)
                .callee(callee_value.into_mlir())
                .callee_operands(&argument_values)
                .gas(gas)
                .value(value)
                .try_call(Attribute::unit(state.mlir()))
        ));
        let status = operation
            .result(0)
            .expect("sol.ext_icall try produces a status result")
            .into();
        let results = (0..result_types.len())
            .map(|index| {
                operation
                    .result(index + 1)
                    .expect("sol.ext_icall try produces a status plus its declared results")
                    .into()
            })
            .collect();
        (status, results, block)
    }
}
