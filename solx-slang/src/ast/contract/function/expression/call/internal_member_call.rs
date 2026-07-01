//!
//! Internal member-function calls.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;

use solx_mlir::Type as AstType;

use crate::ast::analysis::query::member_access_operand::MemberAccessOperand;
use crate::ast::analysis::query::parameter_node_ids::ParameterNodeIds;
use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_arguments::CallArguments;
use crate::ast::emit::emit_expression::EmitExpression;

/// A member call to an internal function.
pub struct InternalMemberCall {
    /// The member access that selected the function.
    pub access: MemberAccessExpression,
    /// The resolved function.
    pub function: FunctionDefinition,
    /// Receiver passed as the first parameter, if any.
    pub receiver: Option<Expression>,
    /// Arguments ordered against the emitted function parameters.
    pub arguments: CallArguments,
}

impl InternalMemberCall {
    /// Classifies a member call to an internal function.
    pub fn from_callee(callee: &Expression, arguments: &ArgumentsDeclaration) -> Option<Self> {
        let Expression::MemberAccessExpression(access) = callee else {
            return None;
        };
        let Some(Definition::Function(function)) = access.member().resolve_to_definition() else {
            return None;
        };
        if function.compute_selector().is_some() {
            return None;
        }
        let parameter_ids = function.parameters().node_ids();
        let operand = access.operand();
        let (receiver, arguments) = if MemberAccessOperand(&operand).is_namespace_qualifier() {
            (
                None,
                CallArguments::for_parameter_ids(arguments, &parameter_ids),
            )
        } else {
            (
                Some(operand),
                CallArguments::after_receiver(arguments, &parameter_ids),
            )
        };
        Some(Self {
            access: access.clone(),
            function,
            receiver,
            arguments,
        })
    }

    /// Emits the internal member call.
    pub fn emit<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let resolved = context.state.resolve_function(self.function.node_id());
        match &self.receiver {
            None => self.arguments.emit_call(resolved, context, block),
            Some(receiver) => {
                let (parameter_self, parameter_rest) = resolved
                    .parameter_types
                    .split_first()
                    .expect("slang validated");
                let BlockAnd {
                    value: self_value,
                    block,
                } = receiver.emit(context, block);
                let self_value = self_value
                    .cast(AstType::new(*parameter_self), context.state, &block)
                    .into_mlir();
                let BlockAnd {
                    value: mut argument_values,
                    block,
                } = self.arguments.emit_as(parameter_rest, context, block);
                argument_values.insert(0, self_value);
                let results = resolved.call(&argument_values, context.state, &block);
                BlockAnd {
                    value: results,
                    block,
                }
            }
        }
    }
}
