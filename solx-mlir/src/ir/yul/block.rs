//!
//! The Yul block: the receiver of a Yul statement and the region-bearing control flow it opens.
//!

use melior::ir::Block as MlirBlock;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::attribute::DenseElementsAttribute;
use melior::ir::operation::OperationLike;
use melior::ir::operation::OperationRef;
use melior::ir::r#type::RankedTensorType;
use ruint::aliases::U256;

use crate::Block;
use crate::Context;
use crate::Type;
use crate::Word;
use crate::ods::yul::FuncReturnOperation;
use crate::ods::yul::SwitchOperation;

/// A `'context`-scoped Yul dialect block: where Yul effects and terminators are appended, and where
/// `yul.if` / `yul.for` / `yul.switch` are opened. The Yul counterpart of [`Block`], separate
/// because the two dialects spell the same statements with different ops.
#[derive(Clone, Copy)]
pub struct YulBlock<'context> {
    /// The wrapped melior block reference, its block-scoped lifetime collapsed to `'context`.
    pub inner: BlockRef<'context, 'context>,
}

impl<'context> YulBlock<'context> {
    /// The `yul.func_return` terminator carrying a function's return words. A Yul function body
    /// takes no appended terminator: it closes on its own lowering, at the `leave` or the
    /// fall-through end that reaches this op.
    pub fn function_return(self, operands: &[Word<'context>], context: &Context<'context>) {
        let operands: Vec<_> = operands.iter().map(|word| word.into_mlir()).collect();
        self.inner.append_operation(mlir_op_build!(
            context,
            FuncReturnOperation.operands(operands.as_slice())
        ));
    }

    /// Opens `yul.switch` on `argument` over `cases` and hands back the default region's entry
    /// block followed by one per case value, in the order given. A switch with no case values is
    /// not this op: the caller emits its default body inline.
    pub fn switch(
        self,
        argument: Word<'context>,
        cases: &[U256],
        context: &Context<'context>,
    ) -> (Self, Vec<Self>) {
        let values: Vec<_> = cases
            .iter()
            .map(|case| Type::yul_word_attribute(case, context.melior).into())
            .collect();
        let cases_attribute = DenseElementsAttribute::new(
            RankedTensorType::new(
                &[cases.len() as u64],
                Type::yul_word(context.melior).into_mlir(),
                None,
            )
            .into(),
            values.as_slice(),
        )
        .expect("a ranked tensor type is shaped");

        let operation = self.inner.append_operation(
            SwitchOperation::builder(context.melior, context.location())
                .results(&[])
                .arg(argument.into_mlir())
                .default_region(Self::entry_region())
                .case_regions(cases.iter().map(|_| Self::entry_region()).collect())
                .cases(cases_attribute.into())
                .build()
                .into(),
        );
        (
            Self::entry_block(&operation, 0),
            (1..=cases.len())
                .map(|index| Self::entry_block(&operation, index))
                .collect(),
        )
    }

    /// The block argument at `index`, a Yul function parameter.
    pub fn argument(self, index: usize) -> Word<'context> {
        Word::from(
            self.inner
                .argument(index)
                .expect("block argument index in range"),
        )
    }

    /// Whether this block already carries a terminator. A `break`, `continue`, or `leave` mid-block
    /// terminates it, and every statement after it is unreachable.
    pub fn is_terminated(self) -> bool {
        self.inner.terminator().is_some()
    }

    /// A region holding one empty entry block, the shape every Yul control-flow region takes.
    fn entry_region() -> Region<'context> {
        let region = Region::new();
        region.append_block(MlirBlock::new(&[]));
        region
    }

    /// The entry block of the operation's region at `index`.
    fn entry_block(operation: &OperationRef<'context, 'context>, index: usize) -> Self {
        Self::from(
            operation
                .region(index)
                .expect("region index in range")
                .first_block()
                .expect("the region was opened with an entry block"),
        )
    }
}

impl<'context, 'block, B> From<B> for YulBlock<'context>
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

impl<'context> From<Block<'context>> for YulBlock<'context> {
    /// The Yul view of a block: the `sol.inline_asm` body is opened as a Sol block and emitted into
    /// as a Yul one.
    fn from(block: Block<'context>) -> Self {
        Self { inner: block.inner }
    }
}

impl<'context> From<YulBlock<'context>> for Block<'context> {
    /// The Sol view of a Yul block, for the insertion cursor the [`Context`] holds.
    fn from(block: YulBlock<'context>) -> Self {
        Self { inner: block.inner }
    }
}
