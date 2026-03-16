//!
//! Break and continue targets for a loop.
//!

use melior::ir::BlockRef;

/// Break and continue targets for a loop.
pub struct LoopTarget<'context, 'block> {
    /// Block to branch to on `break`.
    break_block: BlockRef<'context, 'block>,
    /// Block to branch to on `continue`.
    continue_block: BlockRef<'context, 'block>,
}

impl<'context, 'block> LoopTarget<'context, 'block> {
    /// Creates a new loop target with the given break and continue blocks.
    pub fn new(
        break_block: BlockRef<'context, 'block>,
        continue_block: BlockRef<'context, 'block>,
    ) -> Self {
        Self {
            break_block,
            continue_block,
        }
    }

    /// Returns the block to branch to on `break`.
    pub fn break_block(&self) -> BlockRef<'context, 'block> {
        self.break_block
    }

    /// Returns the block to branch to on `continue`.
    pub fn continue_block(&self) -> BlockRef<'context, 'block> {
        self.continue_block
    }
}
