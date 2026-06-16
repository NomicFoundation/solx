//!
//! Crate-wide projection macros: generate `impl Emit` for AST nodes that share
//! one emission body.
//!
//! - [`expression_emit`] — value-producing expression nodes (so identically
//!   emitted nodes — e.g. the decimal and hex integer literals — are written
//!   once);
//! - [`statement_emit`] — its statement counterpart.
//!
//! The ODS op-construction macros (`sol_op!` / `sol_op_build!` / `sol_op_void!`)
//! live with the Builder in `solx-mlir`, imported crate-wide via `#[macro_use]`.
//!

/// Generates `impl Emit` for one or more value-producing expression nodes that
/// share the emission `$body` — so nodes that emit identically (decimal and hex
/// integer literals) state their body once. The closure binds the node
/// (`|node, context, block|`) or omits it when unused (`|context, block|`);
/// `context` is the `&ExpressionContext`. Names resolve against the call site's
/// imports; the [`Value`] output type is referenced by absolute path, so a body
/// may keep its own `melior::ir::Value` import for intermediate values.
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
                ) -> Self::Output {
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
                ) -> Self::Output $body
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
                ) -> Self::Output {
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
                ) -> Self::Output $body
            }
        )+
    };
}
