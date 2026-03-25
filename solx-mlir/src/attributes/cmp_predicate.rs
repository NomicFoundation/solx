//!
//! Sol dialect comparison predicate values.
//!

/// Sol dialect `sol.cmp` predicate values.
///
/// Signedness is carried by the operand type (`ui256` vs `si256`),
/// not the predicate. Numeric values match the Sol MLIR dialect
/// `CmpPredicate` encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum CmpPredicate {
    /// Equal.
    Eq = 0,
    /// Not equal.
    Ne = 1,
    /// Less than.
    Lt = 2,
    /// Less than or equal.
    Le = 3,
    /// Greater than.
    Gt = 4,
    /// Greater than or equal.
    Ge = 5,
}
