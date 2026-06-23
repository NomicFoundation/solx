//!
//! Ahead-of-time classification of an expression used in statement position.
//!

use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::ExpressionStatement;
use slang_solidity_v2::ast::FunctionCallExpression;

/// The emission kind of a discarded expression statement, classified once so dispatch is a single
/// `match`. The variants are mutually exclusive and tested in order (an earlier match wins).
pub enum ExpressionStatementKind {
    /// A `revert(...)` / `revert("reason")` call, with bespoke revert emission.
    RevertCall(FunctionCallExpression),
    /// Any other expression: emit it and discard the value.
    Value(Expression),
}

impl ExpressionStatementKind {
    /// Classifies the statement's expression into its [`ExpressionStatementKind`].
    pub fn from_statement(expression_statement: &ExpressionStatement) -> Self {
        let expression = expression_statement.expression();
        if let Expression::FunctionCallExpression(call) = expression_statement.expression()
            && let Expression::Identifier(identifier) = call.operand()
            && matches!(identifier.resolve_to_built_in(), Some(BuiltIn::Revert))
        {
            return Self::RevertCall(call);
        }
        Self::Value(expression)
    }
}
