//!
//! Pure transformations and predicates on Slang's [`Expression`] AST node.
//!

use num_bigint::BigInt;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::LiteralKind;
use slang_solidity_v2::ast::Type as SlangType;

/// Extension methods on Slang's [`Expression`] AST node.
///
/// An extension trait (NOT a slang API); a `pub trait` per the visibility rule
/// (no `pub(crate)`).
pub trait ExpressionExt {
    /// Peels redundant parenthesisation — single-element tuples — to a
    /// fixpoint, so a parenthesised expression (`(x)`, `((x))`, `(super)`) is
    /// treated like its bare inner form, mirroring how solc discards redundant
    /// parentheses. Returns the expression unchanged when it is not so wrapped.
    fn unwrap_parentheses(self) -> Self;

    /// Whether this expression is a namespace qualifier — a library or import
    /// alias naming a scope (`L` in `L.f(...)`, `M` in `M.f(...)`) — rather than
    /// a runtime value. A member call's qualifier operand contributes no
    /// `self` argument, where a value operand becomes the implicit `self`.
    fn is_namespace_qualifier(&self) -> bool;

    /// The exact integer value this expression folds to, when it is a
    /// compile-time-constant arithmetic/bitwise expression slang typed as a
    /// `Literal` carrying the computed value — `None` otherwise. Only a COMPUTED
    /// expression folds (a bare literal keeps its own lowering arm); a
    /// non-integer rational is excluded, having no integer constant to emit.
    fn folded_constant_value(&self) -> Option<BigInt>;
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

    fn unwrap_parentheses(mut self) -> Self {
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

    fn folded_constant_value(&self) -> Option<BigInt> {
        let is_computed = matches!(
            self,
            Expression::AdditiveExpression(_)
                | Expression::MultiplicativeExpression(_)
                | Expression::ExponentiationExpression(_)
                | Expression::ShiftExpression(_)
                | Expression::BitwiseAndExpression(_)
                | Expression::BitwiseOrExpression(_)
                | Expression::BitwiseXorExpression(_)
                | Expression::PrefixExpression(_)
        );
        if !is_computed {
            return None;
        }
        let SlangType::Literal(literal_type) = self.get_type()? else {
            return None;
        };
        match literal_type.kind() {
            LiteralKind::Integer { value } => Some(value),
            // A hex literal's value is an unsigned `BigUint`; widen to the signed
            // `BigInt` the constant emitter expects.
            LiteralKind::HexInteger { value, .. } => Some(BigInt::from(value)),
            LiteralKind::Rational { value } if value.is_integer() => Some(value.to_integer()),
            _ => None,
        }
    }
}
