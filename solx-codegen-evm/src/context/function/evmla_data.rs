//!
//! The LLVM function EVM legacy assembly data.
//!

use std::collections::BTreeMap;

use crate::context::function::block::Block;
use crate::context::function::block::key::Key as BlockKey;

///
/// The LLVM function EVM legacy assembly data.
///
/// Describes some data that is only relevant to the EVM legacy assembly.
///
#[derive(Debug)]
pub struct EVMLAData<'ctx> {
    /// The ordinary blocks with numeric tags.
    /// Is only used by the Solidity EVM compiler.
    pub blocks: BTreeMap<BlockKey, Vec<Block<'ctx>>>,
    /// The function stack size.
    pub stack_size: usize,
}

impl<'ctx> EVMLAData<'ctx> {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(stack_size: usize) -> Self {
        Self {
            blocks: BTreeMap::new(),
            stack_size,
        }
    }

    ///
    /// Inserts a function block.
    ///
    pub fn insert_block(&mut self, key: BlockKey, block: Block<'ctx>) {
        if let Some(blocks) = self.blocks.get_mut(&key) {
            blocks.push(block);
        } else {
            self.blocks.insert(key, vec![block]);
        }
    }
}
