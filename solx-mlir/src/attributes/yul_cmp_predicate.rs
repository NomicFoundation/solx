//!
//! Yul dialect comparison predicate values.
//!

use melior::ir::attribute::IntegerAttribute;
use melior::ir::r#type::IntegerType;

use solx_utils::BIT_LENGTH_X64;

/// Yul dialect `yul.cmp` predicate values — the predicate names the comparison directly,
/// since every Yul word is the signless `i256`.
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

impl YulCmpPredicate {
    /// This predicate's encoding as the `i64` [`IntegerAttribute`] the `yul.cmp`
    /// predicate operand demands.
    pub fn attribute(self, context: &melior::Context) -> IntegerAttribute<'_> {
        IntegerAttribute::new(
            IntegerType::new(context, BIT_LENGTH_X64 as u32).into(),
            self as i64,
        )
    }
}
