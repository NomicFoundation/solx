//!
//! Calls redirected through `super` or a base-contract qualifier.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::NodeId;

use crate::ast::BlockAnd;
use crate::ast::contract::contract_dispatch::ContractDispatch;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_arguments::CallArguments;

/// A function call redirected by inherited dispatch metadata.
pub struct InheritedFunctionCall {
    /// The member access that selected the inherited function.
    pub access: MemberAccessExpression,
    /// The redirected target function ID.
    pub target_id: NodeId,
    /// Arguments ordered against the target parameters.
    pub arguments: CallArguments,
}

impl InheritedFunctionCall {
    /// Classifies a `super.f(...)` or `Base.f(...)` call.
    pub fn from_callee(
        callee: &Expression,
        arguments: &ArgumentsDeclaration,
        dispatch: &ContractDispatch,
    ) -> Option<Self> {
        let Expression::MemberAccessExpression(access) = callee else {
            return None;
        };
        if !matches!(access.operand(), Expression::SuperKeyword(_))
            && dispatch.resolve_super(access.node_id()).is_none()
        {
            return None;
        }
        let target_id = dispatch
            .resolve_super(access.node_id())
            .expect("a super/base call has a recorded redirect target");
        let parameter_ids: Vec<NodeId> = match access.member().resolve_to_definition() {
            Some(Definition::Function(function_definition)) => function_definition
                .parameters()
                .iter()
                .map(|parameter| parameter.node_id())
                .collect(),
            _ => unreachable!("a super/base call resolves its member to a function"),
        };
        Some(Self {
            access: access.clone(),
            target_id,
            arguments: CallArguments::for_parameter_ids(arguments, &parameter_ids),
        })
    }

    /// Emits the inherited call.
    pub fn emit<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let function = context.state.resolve_function(self.target_id);
        let BlockAnd {
            value: argument_values,
            block,
        } = self
            .arguments
            .emit_as(&function.parameter_types, context, block);
        let results = function.call(&argument_values, context.state, &block);
        BlockAnd {
            value: results,
            block,
        }
    }
}
