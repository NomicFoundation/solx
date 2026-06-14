//!
//! Crate-wide macros.
//!
//! - [`sol_op_build`] / [`sol_op`] / [`sol_op_void`] — the ODS-construction
//!   bridge (§2b): inline Sol dialect op construction, used inside the entity or
//!   node that owns the op.
//! - [`expression_emit`] — generates `impl Emit` for value-producing expression
//!   nodes that share one lowering body (so identically-lowered nodes — e.g. the
//!   decimal and hex integer literals — are written once).
//!
//! An op is constructed inline via `<Op>::builder(…)`; these macros collapse the
//! surrounding ceremony — the `(context, unknown_location)` head and the
//! `.build().into()` tail — so only the op name and its field setters stay on
//! screen. They re-spell nothing: the expansion *is* the inline ODS construction
//! (there is no `emit_sol_*` Builder layer that a call site reaches for instead).
//!
//! [`sol_op_build`] is the core (build the `Operation`); [`sol_op`] and
//! [`sol_op_void`] layer the append (and single-result extraction) on top of it,
//! so the builder construction is written exactly once:
//! - [`sol_op_build`] — yield the `Operation`, do not append (for a `match` arm
//!   / closure that hands the op to a shared append site);
//! - [`sol_op`] — append to a block and return the single result value;
//! - [`sol_op_void`] — append a value-less effect op.
//!
//! Ops with `operand_segment_sizes` (`Encode`, `New`, `Emit`) or multiple
//! results (`Decode`) are built by hand — their construction is not a fixed
//! method chain.
//!

/// Builds an inlined dialect op and yields it as an `Operation`, without
/// appending. The op-builder method chain is written inline after the op name.
/// The setter repetition is `*` (not `+`): a field-less op (`sol.break`,
/// `sol.continue`) is written as the bare op name with no setters.
macro_rules! sol_op_build {
    ($builder:expr, $operation:ident $(.$method:ident($($argument:expr),* $(,)?))*) => {
        $operation::builder($builder.context, $builder.unknown_location)
            $(.$method($($argument),*))*
            .build()
            .into()
    };
}

/// Builds an inlined dialect op ([`sol_op_build!`]), appends it to `$block`, and
/// returns its single result value. The `expect` message is derived from the op.
macro_rules! sol_op {
    ($builder:expr, $block:expr, $operation:ident $(.$method:ident($($argument:expr),* $(,)?))*) => {
        $block
            .append_operation(sol_op_build!($builder, $operation $(.$method($($argument),*))*))
            .result(0)
            .expect(concat!(stringify!($operation), " produces one result"))
            .into()
    };
}

/// [`sol_op!`] for a value-less op — a statement / effect such as `sol.transfer`
/// or `sol.log`: appends the op ([`sol_op_build!`]) and yields `()`.
macro_rules! sol_op_void {
    ($builder:expr, $block:expr, $operation:ident $(.$method:ident($($argument:expr),* $(,)?))*) => {
        $block.append_operation(sol_op_build!($builder, $operation $(.$method($($argument),*))*));
    };
}

/// Generates `impl Emit` for one or more value-producing expression nodes that
/// share the emission `$body` — so nodes that emit identically (the decimal and
/// hex integer literals) state their body once. The closure binds the node
/// (`|node, context, block|`, where `node` is the `&self` AST node) or omits it
/// when unused (`|context, block|`); `context` is the `&ExpressionContext`. The
/// body returns `anyhow::Result<BlockAnd<Value>>` (an expression in value position
/// always produces a value). Names resolve against the call site's imports
/// (`Emit`, `BlockAnd`, `ExpressionContext`, `BlockRef`); the [`Value`] output
/// type is referenced by absolute path, so a body may keep its own
/// `melior::ir::Value` import for intermediate values.
macro_rules! expression_emit {
    ($($node:ty),+ ; |$bound:ident, $context:ident, $block:ident| $body:block) => {
        $(
            impl<'state, 'context, 'block, 'scope> Emit<'context, 'block, 'state, 'scope> for $node
            where
                'context: 'block,
                'context: 'state,
                'block: 'state,
                'state: 'scope,
            {
                type Context = &'scope ExpressionContext<'state, 'context, 'block>;
                type Output = BlockAnd<'context, 'block, $crate::ast::Value<'context, 'block>>;

                fn emit(
                    &self,
                    $context: Self::Context,
                    $block: BlockRef<'context, 'block>,
                ) -> anyhow::Result<Self::Output> {
                    let $bound = self;
                    $body
                }
            }
        )+
    };
    ($($node:ty),+ ; |$context:ident, $block:ident| $body:block) => {
        $(
            impl<'state, 'context, 'block, 'scope> Emit<'context, 'block, 'state, 'scope> for $node
            where
                'context: 'block,
                'context: 'state,
                'block: 'state,
                'state: 'scope,
            {
                type Context = &'scope ExpressionContext<'state, 'context, 'block>;
                type Output = BlockAnd<'context, 'block, $crate::ast::Value<'context, 'block>>;

                fn emit(
                    &self,
                    $context: Self::Context,
                    $block: BlockRef<'context, 'block>,
                ) -> anyhow::Result<Self::Output> $body
            }
        )+
    };
}

/// `expression_emit!`'s statement counterpart: generates `impl Emit` for one or
/// more statement nodes. The context is `&mut StatementContext` (a statement may
/// declare variables) and the output is `Option<BlockRef>` — the continuation
/// block, or `None` when control diverged (`return` / `break` / `continue`). The
/// closure binds the node (`|node, context, block|`) or omits it when unused
/// (`|context, block|`). Names resolve against the call site's imports.
macro_rules! statement_emit {
    ($($node:ty),+ ; |$bound:ident, $context:ident, $block:ident| $body:block) => {
        $(
            impl<'state, 'context, 'block, 'scope> Emit<'context, 'block, 'state, 'scope> for $node
            where
                'context: 'block,
                'context: 'state,
                'block: 'state,
                'state: 'scope,
            {
                type Context = &'scope mut StatementContext<'state, 'context, 'block>;
                type Output = Option<BlockRef<'context, 'block>>;

                fn emit(
                    &self,
                    $context: Self::Context,
                    $block: BlockRef<'context, 'block>,
                ) -> anyhow::Result<Self::Output> {
                    let $bound = self;
                    $body
                }
            }
        )+
    };
    ($($node:ty),+ ; |$context:ident, $block:ident| $body:block) => {
        $(
            impl<'state, 'context, 'block, 'scope> Emit<'context, 'block, 'state, 'scope> for $node
            where
                'context: 'block,
                'context: 'state,
                'block: 'state,
                'state: 'scope,
            {
                type Context = &'scope mut StatementContext<'state, 'context, 'block>;
                type Output = Option<BlockRef<'context, 'block>>;

                fn emit(
                    &self,
                    $context: Self::Context,
                    $block: BlockRef<'context, 'block>,
                ) -> anyhow::Result<Self::Output> $body
            }
        )+
    };
}
