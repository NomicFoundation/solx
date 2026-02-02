//!
//! The LLVM IR EVMLA function trait.
//!

use crate::context::function::block::Block;
use crate::context::function::block::key::Key as BlockKey;

///
/// The LLVM IR EVMLA function trait.
///
pub trait IEVMLAFunction<'ctx> {
    ///
    /// Returns the block with the specified tag and initial stack pattern.
    ///
    /// If there is only one block, it is returned unconditionally.
    ///
    fn find_block(&self, key: &BlockKey, stack_hash: &u64) -> anyhow::Result<Block<'ctx>>;
}
