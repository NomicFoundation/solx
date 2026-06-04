//!
//! Tuple expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Expression;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Evaluates an expression into one value per tuple component.
    ///
    /// A tuple of more than one component (`(a, b)`) yields its component
    /// values; a single-component tuple is a parenthesized expression, and any
    /// other expression yields a single value. Shared by multi-value returns
    /// and tuple deconstruction.
    pub fn emit_component_values(
        &self,
        expression: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let Expression::TupleExpression(tuple) = expression else {
            let (value, block) = self.emit_value(expression, block)?;
            return Ok((vec![value], block));
        };
        let items = tuple.items();
        if items.len() <= 1 {
            let (value, block) = self.emit_value(expression, block)?;
            return Ok((vec![value], block));
        }

        let mut values = Vec::with_capacity(items.len());
        let mut block = block;
        for item in items.iter() {
            let component = item
                .expression()
                .expect("a multi-component tuple has no empty components");
            let (value, next_block) = self.emit_value(&component, block)?;
            values.push(value);
            block = next_block;
        }
        Ok((values, block))
    }
}
