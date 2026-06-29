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
use slang_solidity_v2::ast::NodeId;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::Type as AstType;
use crate::ast::analysis::query::MemberAccessOperand;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_arguments::CallArguments;

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
        let parameter_ids: Vec<NodeId> = function
            .parameters()
            .iter()
            .map(|parameter| parameter.node_id())
            .collect();
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
            None => {
                let BlockAnd {
                    value: argument_values,
                    block,
                } = self
                    .arguments
                    .emit_as(&resolved.parameter_types, context, block);
                let results = resolved.call(&argument_values, context.state, &block);
                BlockAnd {
                    value: results,
                    block,
                }
            }
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
