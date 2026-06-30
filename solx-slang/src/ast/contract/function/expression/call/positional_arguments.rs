//!
//! Call-argument list emission: the raw positional list and the coerced,
//! parameter-ordered list a call passes to its callee.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::BlockAnd;
use crate::ast::EmitAs;
use crate::ast::EmitExpression;
use crate::ast::contract::function::expression::ExpressionContext;

impl<'context: 'block, 'block> EmitExpression<'context, 'block> for PositionalArguments {
    type Output = BlockAnd<'context, 'block, Vec<Value<'context, 'block>>>;

    /// Evaluates each positional argument left-to-right into its raw, uncoerced value.
    fn emit<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> Self::Output {
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

impl<'types, 'context: 'block, 'block> EmitAs<'context, 'block, &'types [Type<'context>]>
    for [Expression]
{
    type Output = Vec<Value<'context, 'block>>;

    /// Evaluates an ordered argument list, coercing each argument to its declared parameter type:
    /// the shared eval-and-coerce primitive, with the list arriving already in parameter order.
    fn emit_as<'state>(
        &self,
        parameter_types: &'types [Type<'context>],
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let mut values = Vec::with_capacity(self.len());
        let mut block = block;
        for (argument, &parameter_type) in self.iter().zip(parameter_types) {
            let BlockAnd { value, block: next } = argument.emit_as(parameter_type, context, block);
            values.push(value.into_mlir());
            block = next;
        }
        BlockAnd {
            value: values,
            block,
        }
    }
}
