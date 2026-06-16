//!
//! Positional call-argument list emission.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::contract::function::expression::ExpressionContext;

impl<'state, 'context, 'block, 'scope> Emit<'context, 'block, 'state, 'scope>
    for PositionalArguments
where
    'context: 'block,
    'context: 'state,
    'block: 'state,
    'state: 'scope,
{
    type Context = &'scope ExpressionContext<'state, 'context, 'block>;
    type Output = BlockAnd<'context, 'block, Vec<Value<'context, 'block>>>;

    /// Evaluates each positional argument left-to-right into its raw value. The
    /// values are uncoerced — a call coerces each to its callee's parameter type,
    /// a built-in to its operand type.
    fn emit(&self, context: Self::Context, block: BlockRef<'context, 'block>) -> Self::Output {
        let mut values = Vec::with_capacity(self.len());
        let mut block = block;
        for argument in self.iter() {
            let BlockAnd { value, block: next } = argument.emit(context, block);
            values.push(value.into_mlir());
            block = next;
        }
        BlockAnd {
            value: values,
            block,
        }
    }
}
