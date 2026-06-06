//!
//! `ExpressionExt::unwrap_parens` extension trait.
//!

use slang_solidity_v2::ast::Expression;

/// Extension methods on Slang's [`Expression`] AST node.
///
/// The recut's own extension trait (NOT a slang API), promoted from the oracle's
/// `pub(crate)` helper to a `pub trait` per the recut rules (no `pub(crate)`).
pub trait ExpressionExt {
    /// Peels redundant parenthesisation — single-element tuples — to a
    /// fixpoint, so a parenthesised expression (`(x)`, `((x))`, `(super)`) is
    /// treated like its bare inner form, mirroring how solc discards redundant
    /// parentheses. Returns the expression unchanged when it is not so wrapped.
    fn unwrap_parens(self) -> Self;
}

impl ExpressionExt for Expression {
    fn unwrap_parens(mut self) -> Self {
        loop {
            let inner = match &self {
                Expression::TupleExpression(tuple) if tuple.items().len() == 1 => tuple
                    .items()
                    .iter()
                    .next()
                    .and_then(|item| item.expression()),
                _ => None,
            };
            match inner {
                Some(next) => self = next,
                None => return self,
            }
        }
    }
}
