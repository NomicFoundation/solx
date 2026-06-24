//!
//! User-defined operator binding key.
//!

/// A Solidity operator bindable to a function via `using {f as op} for T global;`.
///
/// Binary `-` ([`Self::Sub`]) and unary `-` ([`Self::Neg`]) are distinct variants — the same token
/// binds to a two-parameter function as subtraction and a one-parameter function as negation.
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
