//!
//! A produced value paired with the block emission continues in.
//!

use melior::ir::BlockRef;

/// A produced `T` paired with the MLIR block subsequent operations append to.
///
/// MLIR blocks are values, so emission threads the current block explicitly instead of mutating an
/// insertion point.
pub struct BlockAnd<'context, 'block, T> {
    /// The continuation block subsequent operations are appended to.
    pub block: BlockRef<'context, 'block>,
    /// The produced value.
    pub value: T,
}
