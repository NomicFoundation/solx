//!
//! Yul dialect comparison predicates.
//!

predicate_attribute! {
    /// Yul dialect `yul.cmp` predicate values. Yul is signless, so the predicate alone carries the
    /// signedness the builtin's name spells out.
    YulCmpPredicate {
        /// Equal.
        Eq = 0,
        /// Not equal.
        Ne = 1,
        /// Unsigned less than.
        UnsignedLt = 2,
        /// Unsigned less than or equal.
        UnsignedLe = 3,
        /// Unsigned greater than.
        UnsignedGt = 4,
        /// Unsigned greater than or equal.
        UnsignedGe = 5,
        /// Signed less than.
        SignedLt = 6,
        /// Signed less than or equal.
        SignedLe = 7,
        /// Signed greater than.
        SignedGt = 8,
        /// Signed greater than or equal.
        SignedGe = 9,
    }
}
