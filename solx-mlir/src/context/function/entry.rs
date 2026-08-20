//!
//! A defined `sol.func`'s entry block paired with its declared dispatch.
//!

use crate::Block;
use crate::FunctionDispatch;

/// A defined `sol.func`'s entry block, paired with the dispatch its declaration carries, so the
/// body's lowering never restates it.
#[derive(Clone, Copy)]
pub struct FunctionEntry<'context> {
    /// The entry block the function body is emitted into.
    pub block: Block<'context>,
    /// The dispatch the `sol.func` was declared with.
    pub dispatch: FunctionDispatch,
}

impl<'context> FunctionEntry<'context> {
    /// Pairs a defined function's entry block with its declared dispatch.
    pub fn new(block: Block<'context>, dispatch: FunctionDispatch) -> Self {
        Self { block, dispatch }
    }
}
