//!
//! Emission of a call's positional argument list.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::call::CallContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::emit::emit_expression::EmitExpression;

impl<'emitter, 'state, 'context, 'block> CallContext<'emitter, 'state, 'context, 'block> {
    /// Emits each positional argument and returns the resulting values
    /// alongside the current block.
    pub(super) fn emit_argument_values(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> (Vec<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        let mut values = Vec::with_capacity(arguments.len());
        let mut current = block;
        for argument in arguments.iter() {
            let BlockAnd { value, block: next } = argument.emit(self.expression_context, current);
            values.push(value);
            current = next;
        }
        (values, current)
    }

    /// Orders `arguments` against `parameter_ids`, emits each, and casts it to the matching declared
    /// `parameter_types`, returning the values alongside the current block.
    pub(super) fn emit_ordered_arguments(
        &self,
        arguments: &ArgumentsDeclaration,
        parameter_ids: &[NodeId],
        parameter_types: &[Type<'context>],
        block: BlockRef<'context, 'block>,
    ) -> (Vec<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        let context = self.expression_context.state;
        let ordered_arguments = arguments
            .ordered_by(parameter_ids)
            .expect("slang matches every call argument to a parameter");
        let mut values = Vec::with_capacity(ordered_arguments.len());
        let mut current = block;
        for (argument, &parameter_type) in ordered_arguments.iter().zip(parameter_types) {
            let BlockAnd { value, block: next } = argument.emit(self.expression_context, current);
            let value =
                TypeConversion::from_target_type(parameter_type, context).emit(value, context, &next);
            values.push(value);
            current = next;
        }
        (values, current)
    }
}
