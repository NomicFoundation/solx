//!
//! Tuple expressions: parenthesized values and multi-value positions.
//!

use slang_solidity_v2::ast::TupleExpression;

use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// A parenthesized expression; the multi-element form only yields values in multi-value
    /// positions.
    pub fn tuple(&mut self, node: &TupleExpression) -> Value<'context> {
        if node.items().len() != 1 {
            unimplemented!("a multi-element tuple in single-value position is not yet supported");
        }
        self.tuple_values(node).pop().expect("length checked")
    }

    /// A tuple's elements in declaration order.
    pub fn tuple_values(&mut self, node: &TupleExpression) -> Vec<Value<'context>> {
        node.items()
            .iter()
            .map(|item| {
                self.expression(&item.expression().expect("slang validates tuple elements"))
            })
            .collect()
    }
}
