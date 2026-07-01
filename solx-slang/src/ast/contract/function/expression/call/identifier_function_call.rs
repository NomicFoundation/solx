//!
//! Calls whose callee is an identifier resolving to a function.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;

use crate::ast::analysis::query::node_ids::ParameterNodeIds;
use crate::ast::block_and::BlockAnd;
use crate::ast::contract::contract_dispatch::ContractDispatch;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_arguments::CallArguments;

/// A direct identifier function call after virtual dispatch resolution.
pub struct IdentifierFunctionCall {
    /// The target function ID after virtual dispatch resolution.
    pub target_id: NodeId,
    /// Arguments ordered against the function parameters.
    pub arguments: CallArguments,
}

impl IdentifierFunctionCall {
    /// Classifies an identifier function call.
    pub fn from_callee(
        callee: &Expression,
        arguments: &ArgumentsDeclaration,
        dispatch: &ContractDispatch,
    ) -> Option<Self> {
        let Expression::Identifier(identifier) = callee else {
            return None;
        };
        let Some(Definition::Function(function_definition)) = identifier.resolve_to_definition()
        else {
            return None;
        };
        let parameter_ids = function_definition.parameters().node_ids();
        Some(Self {
            target_id: dispatch.resolve_virtual(function_definition.node_id()),
            arguments: CallArguments::for_parameter_ids(arguments, &parameter_ids),
        })
    }

    /// Emits the function call.
    pub fn emit<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let function = context.state.resolve_function(self.target_id);
        self.arguments.emit_call(function, context, block)
    }
}
