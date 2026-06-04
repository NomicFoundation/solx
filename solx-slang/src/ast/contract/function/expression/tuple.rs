//!
//! Tuple expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::TupleExpression;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers a tuple expression in value position.
    ///
    /// A single-element tuple is a parenthesized expression and lowers to its
    /// inner value; multi-value tuples (only meaningful as assignment targets)
    /// are not handled here.
    pub fn emit_tuple(
        &self,
        tuple: &TupleExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let items = tuple.items();
        if items.len() != 1 {
            unimplemented!("multi-value tuples in value position are not yet supported");
        }
        let item = items.iter().next().expect("length checked to be 1 above");
        let inner = item
            .expression()
            .expect("a tuple element wraps an expression");
        self.emit(&inner, block)
    }
}
