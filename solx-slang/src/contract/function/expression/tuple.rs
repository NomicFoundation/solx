//!
//! Tuple expressions: parenthesized values and multi-value positions.
//!

use slang_solidity_v2::ast::TupleExpression;

use solx_mlir::Place;
use solx_mlir::Type;
use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// A parenthesized value: the single value it wraps. A multi-element tuple is a value only in
    /// multi-value positions, never here.
    pub fn tuple(&mut self, node: &TupleExpression) -> Value<'context> {
        self.tuple_values(node)
            .into_iter()
            .next()
            .expect("a parenthesized value wraps an operand")
    }

    /// A tuple's elements in declaration order, each contributing every value it yields, so a nested
    /// tuple flattens into the same list.
    pub fn tuple_values(&mut self, node: &TupleExpression) -> Vec<Value<'context>> {
        node.items()
            .iter()
            .flat_map(|item| {
                self.expression_values(&item.expression().expect("slang validates tuple elements"))
            })
            .collect()
    }

    /// A tuple's assignment targets in declaration order, each contributing every place it denotes, so
    /// a nested tuple flattens into the same list; a blank element denotes no place and discards the
    /// value paired with it.
    pub fn tuple_places(
        &mut self,
        node: &TupleExpression,
    ) -> Vec<Option<(Place<'context>, Type<'context>)>> {
        node.items()
            .iter()
            .flat_map(|item| match item.expression() {
                Some(target) => self.expression_places(&target),
                None => vec![None],
            })
            .collect()
    }

    /// A tuple in statement position: each element is evaluated for its side effects.
    pub fn tuple_effect(&mut self, node: &TupleExpression) {
        for item in node.items().iter() {
            self.expression_effect(&item.expression().expect("slang validates tuple elements"));
        }
    }
}
