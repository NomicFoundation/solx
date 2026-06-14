//!
//! Pure transformations on Slang's [`TupleExpression`] AST node.
//!

use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::TupleExpression;

/// Extension methods on Slang's [`TupleExpression`] AST node.
///
/// An extension trait (NOT a slang API); a `pub trait` per the visibility rule
/// (no `pub(crate)`).
pub trait TupleExpressionExt {
    /// Pairs this tuple's left-hand-side lvalue slots with `rhs`'s value
    /// expressions for a destructuring assignment, recursing into nested tuples
    /// only where BOTH sides nest — so a blank LHS slot opposite a nested RHS
    /// tuple discards it as a unit. A blank slot yields `None` for its lvalue.
    fn pair_assignment(&self, rhs: &TupleExpression) -> Vec<(Option<Expression>, Expression)>;

    /// Flattens this tuple's left-hand-side leaves, recursing into nested tuples
    /// (`(a, (b, c))` -> `[a, b, c]`). A blank slot is `None` (discarded). Used
    /// for call / conditional right-hand sides, whose values are already flat.
    fn flatten_lvalues(&self) -> Vec<Option<Expression>>;
}

impl TupleExpressionExt for TupleExpression {
    fn pair_assignment(&self, rhs: &TupleExpression) -> Vec<(Option<Expression>, Expression)> {
        let lhs_items = self.items();
        let rhs_items = rhs.items();
        assert!(
            lhs_items.len() == rhs_items.len(),
            "tuple assignment arity mismatch: {} LHS slots vs {} RHS values",
            lhs_items.len(),
            rhs_items.len(),
        );
        let mut pairs = Vec::new();
        for (lhs_item, rhs_item) in lhs_items.iter().zip(rhs_items.iter()) {
            let lhs_expression = lhs_item.expression();
            let rhs_expression = rhs_item
                .expression()
                .expect("a tuple assignment RHS element has an inner expression");
            match (&lhs_expression, &rhs_expression) {
                (
                    Some(Expression::TupleExpression(lhs_nested)),
                    Expression::TupleExpression(rhs_nested),
                ) => {
                    pairs.extend(lhs_nested.pair_assignment(rhs_nested));
                }
                _ => pairs.push((lhs_expression, rhs_expression)),
            }
        }
        pairs
    }

    fn flatten_lvalues(&self) -> Vec<Option<Expression>> {
        let mut leaves = Vec::new();
        for item in self.items().iter() {
            match item.expression() {
                Some(Expression::TupleExpression(nested)) => {
                    leaves.extend(nested.flatten_lvalues());
                }
                other => leaves.push(other),
            }
        }
        leaves
    }
}
