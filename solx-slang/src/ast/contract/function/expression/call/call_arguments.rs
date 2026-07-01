//!
//! Emission of a call's positional argument list.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::call::CallContext;
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
}
