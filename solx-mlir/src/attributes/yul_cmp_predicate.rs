//!
//! Yul dialect comparison predicate values.
//!

/// Yul dialect `yul.cmp` predicate values.
///
/// Unlike `sol.cmp` (which carries signedness in the operand type), the Yul
/// predicate names the comparison directly because every Yul word is the
/// signless `i256`. Numeric values match the Yul MLIR dialect
/// `CmpPredicate` encoding (`YulBase.td`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum YulCmpPredicate {
    /// Equal (`eq`).
    Eq = 0,
    /// Not equal (`ne`).
    Ne = 1,
    /// Unsigned less than (`ult`, Yul `lt`).
    Ult = 2,
    /// Unsigned less than or equal (`ule`).
    Ule = 3,
    /// Unsigned greater than (`ugt`, Yul `gt`).
    Ugt = 4,
    /// Unsigned greater than or equal (`uge`).
    Uge = 5,
    /// Signed less than (`slt`, Yul `slt`).
    Slt = 6,
    /// Signed less than or equal (`sle`).
    Sle = 7,
    /// Signed greater than (`sgt`, Yul `sgt`).
    Sgt = 8,
    /// Signed greater than or equal (`sge`).
    Sge = 9,
}
