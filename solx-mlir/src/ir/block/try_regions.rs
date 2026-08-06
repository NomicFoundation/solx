//!
//! Region entry blocks of `sol::TryOp`.
//!

use crate::Block;

/// The entry blocks a `sol.try` op hands back. Success always exists; an undeclared handler left its
/// region blockless and yields none.
pub struct TryRegions<'context> {
    /// The block reached when the guarded call succeeded, where its results are bound.
    pub success: Block<'context>,
    /// The block of the `catch Panic(uint256)` clause, its code bound as the block argument.
    pub panic: Option<Block<'context>>,
    /// The block of the `catch Error(string memory)` clause, its reason bound as the block argument.
    pub error: Option<Block<'context>>,
    /// The block of the catch-all clause, the revert's return data bound as the block argument when
    /// the clause declares one.
    pub fallback: Option<Block<'context>>,
}
