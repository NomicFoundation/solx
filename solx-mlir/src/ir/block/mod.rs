//!
//! The Block entity: a Sol dialect block, home to the effects and terminators appended to it and the
//! region-bearing control-flow ops it opens.
//!
//! A block is the receiver of a statement the way [`Value`] and [`Place`](crate::Place) are the
//! receivers of an expression. Every block emitted for a contract lives in the module until it is
//! finalized, so its block-scoped lifetime collapses to `'context`: the frontend holds a [`Block`]
//! without naming a block lifetime, and repositions the [`Context`] insertion cursor onto one.
//!

pub mod fallback_region;
pub mod try_regions;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Operation;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::operation::OperationLike;
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
    /// Appends `operation` to this block, returning its reference.
    pub fn append_operation(
        self,
        operation: Operation<'context>,
    ) -> OperationRef<'context, 'context> {
        self.inner.append_operation(operation)
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
        let location = context.location();
        let return_data = Type::string(context.melior, solx_utils::DataLocation::Memory);
        let entry = |arguments: &[Type<'context>]| {
            let region = Region::new();
            let arguments: Vec<_> = arguments
                .iter()
                .map(|argument| (argument.into_mlir(), location))
                .collect();
            region.append_block(melior::ir::Block::new(&arguments));
            region
        };

        let declared_fallback =
            fallback.map(|fallback| entry(fallback.binding(context).as_slice()));
        let operation = self.inner.append_operation(
            TryOperation::builder(context.melior, location)
                .status(status.into_mlir())
                .success_region(entry(&[]))
                .panic_region(if panic {
                    entry(&[Type::field(context.melior)])
                } else {
                    Region::new()
                })
                .error_region(if error {
                    entry(&[return_data])
                } else {
                    Region::new()
                })
                .fallback_region(declared_fallback.unwrap_or_default())
                .build()
                .into(),
        );
        let entry_block = |index: usize| {
            operation
                .region(index)
                .expect("sol.try region index in range")
                .first_block()
                .map(Self::from)
        };
        TryRegions {
            success: entry_block(0).expect("the success region is always declared"),
            panic: entry_block(1),
            error: entry_block(2),
            fallback: entry_block(3),
        }
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
