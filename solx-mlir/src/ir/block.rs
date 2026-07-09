//!
//! The Block entity: a Sol dialect block, home to the effects and terminators appended to it and the
//! region-bearing control-flow ops it opens.
//!
//! A block is the receiver of a statement the way [`Value`] and [`Place`](crate::Place) are the
//! receivers of an expression. Every block emitted for a contract lives in the module until it is
//! finalized, so its block-scoped lifetime collapses to `'context`: the frontend holds a [`Block`]
//! without naming a block lifetime, and repositions the [`Context`] insertion cursor onto one.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Operation;
use melior::ir::Value as MlirValue;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::operation::OperationRef;
use melior::ir::r#type::IntegerType;

use crate::Context;
use crate::Value;
use crate::ods::sol::AssertOperation;
use crate::ods::sol::BreakOperation;
use crate::ods::sol::ConditionOperation;
use crate::ods::sol::ContinueOperation;
use crate::ods::sol::DoWhileOperation;
use crate::ods::sol::EmitOperation;
use crate::ods::sol::ForOperation;
use crate::ods::sol::IfOperation;
use crate::ods::sol::RequireOperation;
use crate::ods::sol::ReturnOperation;
use crate::ods::sol::RevertOperation;
use crate::ods::sol::WhileOperation;
use crate::ods::sol::YieldOperation;

/// A `'context`-scoped Sol dialect block: the insertion point for the effects and terminators
/// appended to it, and the region-bearing control-flow ops it opens.
#[derive(Clone, Copy)]
pub struct Block<'context> {
    /// The wrapped melior block reference, its block-scoped lifetime collapsed to `'context`.
    pub inner: BlockRef<'context, 'context>,
}

impl<'context> Block<'context> {
    /// Emits `sol.emit` with `indexed` topics ahead of `non_indexed` data and the topic count in
    /// `indexedArgsCount`; a named event bakes its `signature`. EVM events carry at most four indexed
    /// topics, so the count fits the dialect's `i8`.
    pub fn emit(
        self,
        signature: Option<&str>,
        indexed: &[Value<'context>],
        non_indexed: &[Value<'context>],
        context: &Context<'context>,
    ) {
        let arguments: Vec<MlirValue<'context, 'context>> = indexed
            .iter()
            .chain(non_indexed.iter())
            .map(|argument| argument.into_mlir())
            .collect();
        let indexed_count =
            i8::try_from(indexed.len()).expect("EVM events carry at most four indexed arguments");
        let indexed_count_attribute = IntegerAttribute::new(
            IntegerType::new(context.melior, 8).into(),
            indexed_count.into(),
        );
        let mut builder = EmitOperation::builder(context.melior, context.location())
            .args(&arguments)
            .indexed_args_count(indexed_count_attribute);
        if let Some(signature) = signature {
            builder = builder.signature(StringAttribute::new(context.melior, signature));
        }
        self.inner.append_operation(builder.build().into());
    }

    /// Emits `sol.require %condition`. A literal `message` bakes into `msg`; an `is_custom_error`
    /// require ABI-encodes its runtime `arguments` under the `Error(string)` selector via the `call`
    /// form.
    pub fn require(
        self,
        condition: Value<'context>,
        arguments: &[Value<'context>],
        message: Option<&str>,
        is_custom_error: bool,
        context: &Context<'context>,
    ) {
        let arguments: Vec<MlirValue<'context, 'context>> = arguments
            .iter()
            .map(|argument| argument.into_mlir())
            .collect();
        let mut builder = RequireOperation::builder(context.melior, context.location())
            .cond(condition.into_mlir())
            .args(arguments.as_slice());
        if let Some(message) = message {
            builder = builder.msg(StringAttribute::new(context.melior, message));
        }
        if is_custom_error {
            builder = builder.call(Attribute::unit(context.melior));
        }
        self.inner.append_operation(builder.build().into());
    }

    /// Emits `sol.assert %condition`.
    pub fn assert(self, condition: Value<'context>, context: &Context<'context>) {
        mlir_op_void!(context, self.inner, AssertOperation.cond(condition));
    }

    /// Emits `sol.revert` carrying `signature` and `args`; `is_custom_error` marks a custom-error
    /// revert with the `call` unit attribute.
    pub fn revert(
        self,
        signature: &str,
        arguments: &[Value<'context>],
        is_custom_error: bool,
        context: &Context<'context>,
    ) {
        let arguments: Vec<MlirValue<'context, 'context>> = arguments
            .iter()
            .map(|argument| argument.into_mlir())
            .collect();
        let mut builder = RevertOperation::builder(context.melior, context.location())
            .signature(StringAttribute::new(context.melior, signature))
            .args(arguments.as_slice());
        if is_custom_error {
            builder = builder.call(Attribute::unit(context.melior));
        }
        self.inner.append_operation(builder.build().into());
    }

    /// Emits `sol.return` carrying `operands`.
    pub fn r#return(self, operands: &[Value<'context>], context: &Context<'context>) {
        let operands = operands
            .iter()
            .map(|operand| operand.into_mlir())
            .collect::<Vec<_>>();
        mlir_op_void!(
            context,
            self.inner,
            ReturnOperation.operands(operands.as_slice())
        );
    }

    /// Emits `sol.break`.
    pub fn r#break(self, context: &Context<'context>) {
        mlir_op_void!(context, self.inner, BreakOperation);
    }

    /// Emits `sol.continue`.
    pub fn r#continue(self, context: &Context<'context>) {
        mlir_op_void!(context, self.inner, ContinueOperation);
    }

    /// Emits `sol.yield` carrying `results`, terminating a region body.
    pub fn r#yield(self, results: &[Value<'context>], context: &Context<'context>) {
        let results = results
            .iter()
            .map(|result| result.into_mlir())
            .collect::<Vec<_>>();
        mlir_op_void!(context, self.inner, YieldOperation.ins(results.as_slice()));
    }

    /// Emits `sol.condition` gating a loop region on `condition`.
    pub fn condition(self, condition: Value<'context>, context: &Context<'context>) {
        mlir_op_void!(context, self.inner, ConditionOperation.condition(condition));
    }

    /// Emits `sol.if` and returns the then-region entry block, plus the else-region entry block
    /// when `with_else`; otherwise the else region is left empty.
    pub fn branch(
        self,
        condition: Value<'context>,
        with_else: bool,
        context: &Context<'context>,
    ) -> (Block<'context>, Option<Block<'context>>) {
        let (then_block, else_block) = mlir_region_op!(
            context,
            &self.inner,
            IfOperation.cond(condition);
            then_region;
            else_region if with_else
        );
        (Block::from(then_block), else_block.map(Block::from))
    }

    /// Emits `sol.for` and returns the condition-, body-, and step-region entry blocks.
    pub fn for_loop(
        self,
        context: &Context<'context>,
    ) -> (Block<'context>, Block<'context>, Block<'context>) {
        let (condition_block, body_block, step_block) =
            mlir_region_op!(context, &self.inner, ForOperation; cond, body, step);
        (
            Block::from(condition_block),
            Block::from(body_block),
            Block::from(step_block),
        )
    }

    /// Emits `sol.while` and returns the condition- and body-region entry blocks.
    pub fn while_loop(self, context: &Context<'context>) -> (Block<'context>, Block<'context>) {
        let (condition_block, body_block) =
            mlir_region_op!(context, &self.inner, WhileOperation; cond, body);
        (Block::from(condition_block), Block::from(body_block))
    }

    /// Emits `sol.do_while` and returns the body- and condition-region entry blocks.
    pub fn do_while(self, context: &Context<'context>) -> (Block<'context>, Block<'context>) {
        let (body_block, condition_block) =
            mlir_region_op!(context, &self.inner, DoWhileOperation; body, cond);
        (Block::from(body_block), Block::from(condition_block))
    }

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
