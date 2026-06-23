//!
//! A produced value paired with the block emission continues in.
//!

use melior::ir::BlockRef;

/// A produced `T` paired with the MLIR block subsequent operations append to.
///
/// MLIR blocks are values, so emission threads the current block explicitly instead of mutating an
/// insertion point (as rustc's MIR builder does).
pub struct BlockAnd<'context, 'block, T> {
    /// The block subsequent operations are appended to (the continuation point).
    pub block: BlockRef<'context, 'block>,
    /// The produced value.
    pub value: T,
}
