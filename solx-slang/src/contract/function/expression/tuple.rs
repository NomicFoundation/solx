//!
//! Tuple expressions: parenthesized values and multi-value positions.
//!

use crate::contract::function::expression::Expression;

codegen!(
    TupleExpression {
        /// A parenthesized expression; the multi-element form only yields values in multi-value
        /// positions.
        -> Value |node, scope| {
            if node.items().len() != 1 {
                unimplemented!(
                    "a multi-element tuple in single-value position is not yet supported"
                );
            }
            Self::emit_values(node, scope).pop().expect("length checked")
        }

        -> Values |node, scope| {
            node.items()
                .iter()
                .map(|item| {
                    let element = item.expression().expect("slang validates tuple elements");
                    Expression::emit(&element, scope)
                })
                .collect()
        }
    }
);
