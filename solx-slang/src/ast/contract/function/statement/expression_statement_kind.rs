//!
//! Ahead-of-time classification of an expression used in statement position.
//!

use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::ConditionalExpression;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::ExpressionStatement;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::Type as SlangType;

/// The emission kind of a discarded expression statement, classified once so dispatch is a single
/// `match`. The variants are mutually exclusive and tested in order (an earlier match wins).
pub enum ExpressionStatementKind {
    /// The modifier `_;` placeholder: hands off to the wrapped body / next stage.
    ModifierPlaceholder,
    /// A `revert(...)` / `revert("reason")` call, with bespoke revert emission.
    RevertCall(FunctionCallExpression),
    /// A bare type-name or `super` reference (`uint256;`, `super;`, the array-type
    /// form `s[7][];`): compile-time only, with no value and no side effect.
    TypeOrSuperNoop,
    /// A discarded type / module reference (`(cond ? M : M).D;`): the type has no
    /// runtime `sol` representation, but its subexpressions may have side effects.
    TypeReference(Expression),
    /// A discarded tuple-valued conditional (`(c ? (1, 2) : (3, 4));`): no single
    /// value, but the condition and selected branch may have side effects.
    TupleConditional(ConditionalExpression),
    /// Any other expression: emit it and discard the value.
    Value(Expression),
}

impl ExpressionStatementKind {
    /// Classifies the statement's expression into its [`ExpressionStatementKind`].
    pub fn from_statement(expression_statement: &ExpressionStatement) -> Self {
        let expression = expression_statement.expression();
        if let Expression::Identifier(identifier) = &expression
            && matches!(
                identifier.resolve_to_built_in(),
                Some(BuiltIn::ModifierUnderscore)
            )
        {
            return Self::ModifierPlaceholder;
        }
        if let Expression::FunctionCallExpression(call) = expression_statement.expression()
            && let Expression::Identifier(identifier) = call.operand()
            && matches!(identifier.resolve_to_built_in(), Some(BuiltIn::Revert))
        {
            return Self::RevertCall(call);
        }
        // A bare type-name / `super` reference is compile-time only, including the array-type form
        // `s[7][];` (a bound-less index access — `a[i]` always has a start).
        let is_type_or_super_noop = match &expression {
            Expression::ElementaryType(_)
            | Expression::TypeExpression(_)
            | Expression::SuperKeyword(_) => true,
            Expression::IndexAccessExpression(index_access) => {
                index_access.start().is_none() && index_access.end().is_none()
            }
            _ => false,
        };
        if is_type_or_super_noop {
            return Self::TypeOrSuperNoop;
        }
        // A member access with no slang type is a compile-time type / module reference, not a
        // runtime value (slang leaves type / module references untyped).
        if let Expression::MemberAccessExpression(access) = &expression
            && access.get_type().is_none()
        {
            return Self::TypeReference(expression);
        }
        // A discarded tuple-valued conditional has no single value, but its branches may have side
        // effects; peel parentheses first.
        if let Expression::ConditionalExpression(conditional) =
            expression_statement.expression().unwrap_parentheses()
            && matches!(conditional.get_type(), Some(SlangType::Tuple(_)))
        {
            return Self::TupleConditional(conditional);
        }
        Self::Value(expression)
    }
}
