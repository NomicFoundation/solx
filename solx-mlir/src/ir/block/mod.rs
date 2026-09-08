//!
//! The Block entity: a Sol dialect block, home to the effects and terminators appended to it and the
//! region-bearing control-flow ops it opens.
//!
//! A block is the receiver of a statement the way [`Value`] and [`Place`](crate::Place) are the
//! receivers of an expression. Every block emitted for a contract lives in the module until it is
//! finalized, so its block-scoped lifetime collapses to `'context`: the frontend holds a [`Block`]
//! without naming a block lifetime, and repositions the [`Context`] insertion cursor onto one. A
//! block [`Block::region`] opens is handed to the op the region builds before it is emitted into.
//!

pub mod fallback_region;
pub mod try_regions;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Operation;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::operation::OperationRef;

use crate::Context;
use crate::Type;
use crate::Value;
use crate::ods::sol::TryOperation;

use self::fallback_region::FallbackRegion;
use self::try_regions::TryRegions;

/// A `'context`-scoped Sol dialect block: the insertion point for the effects and terminators
/// appended to it, and the region-bearing control-flow ops it opens.
#[derive(Clone, Copy)]
pub struct Block<'context> {
    /// The wrapped melior block reference, its block-scoped lifetime collapsed to `'context`.
    pub inner: BlockRef<'context, 'context>,
}

impl<'context> Block<'context> {
    /// Opens a region carrying `arguments` in its entry block, returning the region to hand the
    /// op's builder and the block to emit into.
    pub fn region(
        arguments: &[Type<'context>],
        context: &Context<'context>,
    ) -> (Region<'context>, Self) {
        let arguments: Vec<_> = arguments
            .iter()
            .map(|argument| (argument.into_mlir(), context.location()))
            .collect();
        let region = Region::new();
        let entry = Self::from(region.append_block(melior::ir::Block::new(&arguments)));
        (region, entry)
    }

    /// Appends `operation` to this block, returning its reference.
    pub fn append_operation(
        self,
        operation: Operation<'context>,
    ) -> OperationRef<'context, 'context> {
        self.inner.append_operation(operation)
    }

    /// Inserts `operation` at `position` in this block.
    pub fn insert_operation(self, position: usize, operation: Operation<'context>) {
        self.inner.insert_operation(position, operation);
    }

    /// Opens `sol.try` on `status` and hands back the entry block of every declared region. An
    /// omitted clause leaves its region blockless, which is how the pass tells an absent handler
    /// from one with an empty body: an absent fallback forwards the revert on, an empty one
    /// swallows it.
    pub fn r#try(
        self,
        status: Value<'context>,
        panic: bool,
        error: bool,
        fallback: Option<FallbackRegion>,
        context: &Context<'context>,
    ) -> TryRegions<'context> {
        let (success_region, success) = Self::region(&[], context);
        let (panic_region, panic) = panic
            .then(|| Self::region(&[Type::word(context.melior)], context))
            .unzip();
        let (error_region, error) = error
            .then(|| {
                Self::region(
                    &[Type::string(
                        context.melior,
                        solx_utils::DataLocation::Memory,
                    )],
                    context,
                )
            })
            .unzip();
        let (fallback_region, fallback) = fallback
            .map(|fallback| Self::region(fallback.binding(context).as_slice(), context))
            .unzip();
        self.inner.append_operation(
            TryOperation::builder(context.melior, context.location())
                .status(status.into_mlir())
                .success_region(success_region)
                .panic_region(panic_region.unwrap_or_default())
                .error_region(error_region.unwrap_or_default())
                .fallback_region(fallback_region.unwrap_or_default())
                .build()
                .into(),
        );
        TryRegions {
            success,
            panic,
            error,
            fallback,
        }
    }

    /// The block's arguments, which an entry block carries in parameter order.
    pub fn arguments(self) -> Vec<Value<'context>> {
        (0..self.inner.argument_count())
            .map(|index| self.argument(index))
            .collect()
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
