//!
//! The Block entity: a Sol dialect block, home to the effects and terminators appended to it and the
//! region-bearing control-flow ops it opens.
//!
//! A block is the receiver of a statement the way [`Value`] and [`Place`](crate::Place) are the
//! receivers of an expression. Every block emitted for a contract lives in the module until it is
//! finalized, so its block-scoped lifetime collapses to `'context`: the frontend holds a [`Block`]
//! without naming a block lifetime, and repositions the [`Context`] insertion cursor onto one.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Operation;
use melior::ir::operation::OperationRef;

use crate::Value;

/// A `'context`-scoped Sol dialect block: the insertion point for the effects and terminators
/// appended to it, and the region-bearing control-flow ops it opens.
#[derive(Clone, Copy)]
pub struct Block<'context> {
    /// The wrapped melior block reference, its block-scoped lifetime collapsed to `'context`.
    pub inner: BlockRef<'context, 'context>,
}

impl<'context> Block<'context> {
    /// Appends `operation` to this block, returning its reference.
    pub fn append_operation(
        self,
        operation: Operation<'context>,
    ) -> OperationRef<'context, 'context> {
        self.inner.append_operation(operation)
    }

    /// The block argument at `index`.
    pub fn argument(self, index: usize) -> Value<'context> {
        Value::from(
            self.inner
                .argument(index)
                .expect("block argument index in range"),
        )
    }

    /// Whether this block already carries a terminator.
    pub fn is_terminated(self) -> bool {
        self.inner.terminator().is_some()
    }
}

impl<'context, 'block, B> From<B> for Block<'context>
where
    B: BlockLike<'context, 'block>,
    'context: 'block,
{
    /// Wraps a melior block, laundering its block-scoped lifetime to `'context`.
    fn from(block: B) -> Self {
        Self {
            inner: unsafe { BlockRef::from_raw(block.to_raw()) },
        }
    }
}
