//!
//! User-defined operator binding key.
//!

/// A Solidity operator that can be bound to a function via a
/// `using {f as op} for T global;` directive (user-defined operators).
///
/// Used as the operator component of [`super::Context::operator_bindings`]'s key.
/// Binary `-` ([`Self::Sub`]) and unary `-` ([`Self::Neg`]) are distinct
/// variants because the same `-` token binds to a two-parameter function as a
/// subtraction operator and to a one-parameter function as a negation operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserDefinedOperator {
    /// Binary `+`.
    Add,
    /// Binary `-`.
    Sub,
    /// Binary `*`.
    Mul,
    /// Binary `/`.
    Div,
    /// Binary `%`.
    Rem,
    /// Binary `&`.
    BitAnd,
    /// Binary `|`.
    BitOr,
    /// Binary `^`.
    BitXor,
    /// Binary `==`.
    Eq,
    /// Binary `!=`.
    Ne,
    /// Binary `<`.
    Lt,
    /// Binary `<=`.
    Le,
    /// Binary `>`.
    Gt,
    /// Binary `>=`.
    Ge,
    /// Unary `-`.
    Neg,
    /// Unary `~`.
    BitNot,
}
