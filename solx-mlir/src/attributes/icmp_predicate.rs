//!
//! MLIR LLVM dialect ICmp predicate values.
//!

/// MLIR LLVM dialect `llvm.icmp` predicate values.
///
/// Matches the LLVM `ICmpPredicate` encoding used by the MLIR LLVM dialect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum ICmpPredicate {
    /// Equal.
    Eq = 0,
    /// Not equal.
    Ne = 1,
    /// Signed less than.
    Slt = 2,
    /// Signed less than or equal.
    Sle = 3,
    /// Signed greater than.
    Sgt = 4,
    /// Signed greater than or equal.
    Sge = 5,
    /// Unsigned less than.
    Ult = 6,
    /// Unsigned less than or equal.
    Ule = 7,
    /// Unsigned greater than.
    Ugt = 8,
    /// Unsigned greater than or equal.
    Uge = 9,
}
