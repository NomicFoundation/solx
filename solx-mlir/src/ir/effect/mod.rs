//!
//! The Effect entity: statement and effect ops emitted at an insertion block.
//!
//! The receiver of a statement is the block it lands in, so an [`Effect`] wraps a block and its
//! [`Context`] the way [`Value`](crate::Value) and [`Place`](crate::Place) wrap an SSA value. Its
//! ops `sol.emit`, `sol.require`, `sol.assert`, and `sol.revert` produce no value and continue in
//! the same block; none is a dialect terminator.
//!

pub mod control_flow;

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value as MlirValue;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::r#type::IntegerType;

use crate::Context;
use crate::Value;
use crate::ods::sol::AssertOperation;
use crate::ods::sol::EmitOperation;
use crate::ods::sol::RequireOperation;
use crate::ods::sol::RevertOperation;

/// An insertion point for statement and effect ops. Constructed from the current block, its methods
/// append non-terminator effects and codegen continues in the same block.
#[derive(Clone, Copy)]
pub struct Effect<'a, 'context, 'block> {
    /// The ambient context.
    context: &'a Context<'context>,
    /// The block the effect lands in.
    block: BlockRef<'context, 'block>,
}

impl<'a, 'context, 'block> Effect<'a, 'context, 'block> {
    /// Wraps `block` as the insertion point for effect ops in `context`.
    pub fn new(context: &'a Context<'context>, block: BlockRef<'context, 'block>) -> Self {
        Self { context, block }
    }

    /// Emits `sol.emit` with `indexed` topics ahead of `non_indexed` data and the topic count in
    /// `indexedArgsCount`; a named event bakes its `signature`. EVM events carry at most four
    /// indexed topics, so the count fits the dialect's `i8`.
    pub fn emit(
        self,
        signature: Option<&str>,
        indexed: &[Value<'context, 'block>],
        non_indexed: &[Value<'context, 'block>],
    ) {
        let arguments: Vec<MlirValue<'context, 'block>> = indexed
            .iter()
            .chain(non_indexed.iter())
            .map(|argument| argument.into_mlir())
            .collect();
        let indexed_count =
            i8::try_from(indexed.len()).expect("EVM events carry at most four indexed arguments");
        let indexed_count_attribute = IntegerAttribute::new(
            IntegerType::new(self.context.melior, 8).into(),
            indexed_count.into(),
        );
        let mut builder = EmitOperation::builder(self.context.melior, self.context.location())
            .args(&arguments)
            .indexed_args_count(indexed_count_attribute);
        if let Some(signature) = signature {
            builder = builder.signature(StringAttribute::new(self.context.melior, signature));
        }
        self.block.append_operation(builder.build().into());
    }

    /// Emits `sol.require %condition`. A literal `message` bakes into `msg`; an `is_custom_error`
    /// require ABI-encodes its runtime `arguments` under the `Error(string)` selector via the
    /// `call` form.
    pub fn require(
        self,
        condition: Value<'context, 'block>,
        arguments: &[Value<'context, 'block>],
        message: Option<&str>,
        is_custom_error: bool,
    ) {
        let arguments: Vec<MlirValue<'context, 'block>> = arguments
            .iter()
            .map(|argument| argument.into_mlir())
            .collect();
        let mut builder = RequireOperation::builder(self.context.melior, self.context.location())
            .cond(condition.into_mlir())
            .args(arguments.as_slice());
        if let Some(message) = message {
            builder = builder.msg(StringAttribute::new(self.context.melior, message));
        }
        if is_custom_error {
            builder = builder.call(Attribute::unit(self.context.melior));
        }
        self.block.append_operation(builder.build().into());
    }

    /// Emits `sol.assert %condition`.
    pub fn assert(self, condition: Value<'context, 'block>) {
        mlir_op_void!(self.context, &self.block, AssertOperation.cond(condition));
    }

    /// Emits `sol.revert` carrying `signature` and `args`; `is_custom_error` marks a custom-error
    /// revert with the `call` unit attribute.
    pub fn revert(
        self,
        signature: &str,
        arguments: &[Value<'context, 'block>],
        is_custom_error: bool,
    ) {
        let arguments: Vec<MlirValue<'context, 'block>> = arguments
            .iter()
            .map(|argument| argument.into_mlir())
            .collect();
        let mut builder = RevertOperation::builder(self.context.melior, self.context.location())
            .signature(StringAttribute::new(self.context.melior, signature))
            .args(arguments.as_slice());
        if is_custom_error {
            builder = builder.call(Attribute::unit(self.context.melior));
        }
        self.block.append_operation(builder.build().into());
    }

    /// Whether the block already carries a terminator.
    pub fn is_terminated(self) -> bool {
        self.block.terminator().is_some()
    }
}
