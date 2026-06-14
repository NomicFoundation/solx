//!
//! A produced value paired with the block lowering continues in.
//!

use melior::ir::BlockRef;

/// A produced `T` paired with the MLIR block subsequent operations append to.
///
/// MLIR blocks are values (melior), so lowering threads the current block
/// explicitly instead of mutating an insertion point — rustc's MIR builder
/// threads the current basic block the same way, as `BlockAnd<T>`. An expression
/// emits a `BlockAnd<Option<Value>>` (`None` for a void call); the value-position
/// form is a `BlockAnd<Value>`.
pub struct BlockAnd<'context, 'block, T> {
    /// The block subsequent operations are appended to (the continuation point).
    pub block: BlockRef<'context, 'block>,
    /// The produced value.
    pub value: T,
}
