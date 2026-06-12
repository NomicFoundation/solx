//!
//! Pure transformations and predicates on Slang's [`Expression`] AST node.
//!

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;

/// Extension methods on Slang's [`Expression`] AST node.
///
/// An extension trait (NOT a slang API); a `pub trait` per the visibility rule
/// (no `pub(crate)`).
pub trait ExpressionExt {
    /// Peels redundant parenthesisation — single-element tuples — to a
    /// fixpoint, so a parenthesised expression (`(x)`, `((x))`, `(super)`) is
    /// treated like its bare inner form, mirroring how solc discards redundant
    /// parentheses. Returns the expression unchanged when it is not so wrapped.
    fn unwrap_parens(self) -> Self;

    /// Whether this expression is a namespace qualifier — a library or import
    /// alias naming a scope (`L` in `L.f(...)`, `M` in `M.f(...)`) — rather than
    /// a runtime value. A member call's qualifier operand contributes no
    /// `self` argument, where a value operand becomes the implicit `self`.
    fn is_namespace_qualifier(&self) -> bool;
}

impl ExpressionExt for Expression {
    fn is_namespace_qualifier(&self) -> bool {
        matches!(
            self,
            Expression::Identifier(identifier)
                if matches!(
                    identifier.resolve_to_definition(),
                    Some(
                        Definition::Library(_)
                            | Definition::Import(_)
                            | Definition::ImportedSymbol(_)
                    )
                )
        )
    }

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
