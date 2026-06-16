//!
//! Ahead-of-time classification of an expression used in statement position.
//!

use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::ConditionalExpression;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::ExpressionStatement;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::Type as SlangType;

/// The emission kind of a discarded expression statement, computed once ahead of
/// emission so dispatch is a single `match` rather than a chain of guards.
///
/// Solidity evaluates an expression statement for its side effects and discards
/// its value, but several shapes need bespoke handling rather than a plain
/// value-discard. The variants are mutually exclusive and tested in order — an
/// earlier match wins (a `revert(...)` is a revert, not a generic value). Each
/// variant owns the slang node its emission needs (slang `Expression` is not
/// `Clone`; the handles are cheap and obtained from the statement).
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
    /// slang accessors hand out fresh owned expression handles cheaply, so each
    /// shape is examined from its own handle and the chosen variant keeps the
    /// node it needs.
    pub fn from_statement(expression_statement: &ExpressionStatement) -> Self {
        let expression = expression_statement.expression();
        // A bare `_;` inside a modifier body is the placeholder for the wrapped
        // body (or the next modifier stage).
        if let Expression::Identifier(identifier) = &expression
            && matches!(
                identifier.resolve_to_built_in(),
                Some(BuiltIn::ModifierUnderscore)
            )
        {
            return Self::ModifierPlaceholder;
        }
        // A `revert(...)` / `revert("reason")` call has bespoke revert emission.
        if let Expression::FunctionCallExpression(call) = expression_statement.expression()
            && let Expression::Identifier(identifier) = call.operand()
            && matches!(identifier.resolve_to_built_in(), Some(BuiltIn::Revert))
        {
            return Self::RevertCall(call);
        }
        // A bare type-name or `super` reference is compile-time only: `uint256;`,
        // `super;`, and the array-type form `s[7][];` (a bound-less index access:
        // `a[i]` always has a start, `a[i:j]` / `a[:j]` a bound).
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
        // A member access with no slang type is a compile-time type / module
        // reference (`M.D`, `(cond ? M : M).D`, `C.S`), not a runtime value: slang
        // types a value-returning member access (a field, getter, `.length`, a
        // built-in like `block.timestamp`) but leaves a type / module reference
        // untyped.
        if let Expression::MemberAccessExpression(access) = &expression
            && access.get_type().is_none()
        {
            return Self::TypeReference(expression);
        }
        // A discarded tuple-valued conditional has no single value, but its
        // condition and selected branch may have side effects. The statement is
        // usually parenthesised (a single-element tuple), so peel those first.
        if let Expression::ConditionalExpression(conditional) =
            expression_statement.expression().unwrap_parentheses()
            && matches!(conditional.get_type(), Some(SlangType::Tuple(_)))
        {
            return Self::TupleConditional(conditional);
        }
        Self::Value(expression)
    }
}
