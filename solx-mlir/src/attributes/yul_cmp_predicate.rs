//!
//! Yul dialect comparison predicate values.
//!

sol_predicate_attribute! {
    /// Yul dialect `yul.cmp` predicate values — the predicate names the comparison directly,
    /// since every Yul word is the signless `i256`.
    YulCmpPredicate {
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
}
