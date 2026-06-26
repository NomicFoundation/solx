//!
//! Crate-wide projection macros: generate the per-family emission impl for AST
//! nodes that share one emission body.
//!
//! - [`expression_emit`]: `impl EmitExpression` for value-producing expression
//!   nodes (so identically emitted nodes: e.g. the decimal and hex integer
//!   literals: are written once);
//! - [`statement_emit`]: its `impl EmitStatement` counterpart;
//! - [`yul_emit`]: the inline-assembly `impl EmitYul` counterpart, threading a
//!   `&mut YulContext` and an explicit per-node output.
//!
//! The ODS op-construction macros (`mlir_op!` / `mlir_op_build!` / `mlir_op_void!`)
//! live with the Builder in `solx-mlir`, imported crate-wide via `#[macro_use]`.
//!

/// Generates `impl EmitExpression` for one or more value-producing expression
/// nodes that share the emission `$body`: so nodes that emit identically (decimal
/// and hex integer literals) state their body once. The closure binds the node
/// (`|node, context, block|`) or omits it when unused (`|context, block|`);
/// `context` is the `&ExpressionContext`. Names resolve against the call site's
/// imports; the [`Value`] output type is referenced by absolute path, so a body
/// may keep its own `melior::ir::Value` import for intermediate values.
macro_rules! expression_emit {
    ($($node:ty),+ ; |$bound:ident, $context:ident, $block:ident| $body:block) => {
        $(
            impl<'context: 'block, 'block> EmitExpression<'context, 'block> for $node {
                type Output = BlockAnd<'context, 'block, $crate::ast::Value<'context, 'block>>;

                fn emit<'state>(
                    &self,
                    $context: &ExpressionContext<'state, 'context, 'block>,
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
            impl<'context: 'block, 'block> EmitExpression<'context, 'block> for $node {
                type Output = BlockAnd<'context, 'block, $crate::ast::Value<'context, 'block>>;

                fn emit<'state>(
                    &self,
                    $context: &ExpressionContext<'state, 'context, 'block>,
                    $block: BlockRef<'context, 'block>,
                ) -> Self::Output $body
            }
        )+
    };
}

/// `expression_emit!`'s statement counterpart: generates `impl EmitStatement` for
/// one or more statement nodes. The context is `&mut StatementContext` (a statement
/// may declare variables) and the output is the fixed `Option<BlockRef>`: the
/// continuation block, or `None` when control diverged (`return` / `break` /
/// `continue`). The closure binds the node (`|node, context, block|`) or omits it
/// when unused (`|context, block|`). Names resolve against the call site's imports.
macro_rules! statement_emit {
    ($($node:ty),+ ; |$bound:ident, $context:ident, $block:ident| $body:block) => {
        $(
            impl<'context: 'block, 'block> EmitStatement<'context, 'block> for $node {
                fn emit<'state>(
                    &self,
                    $context: &mut StatementContext<'state, 'context, 'block>,
                    $block: BlockRef<'context, 'block>,
                ) -> Option<BlockRef<'context, 'block>> {
                    let $bound = self;
                    $body
                }
            }
        )+
    };
    ($($node:ty),+ ; |$context:ident, $block:ident| $body:block) => {
        $(
            impl<'context: 'block, 'block> EmitStatement<'context, 'block> for $node {
                fn emit<'state>(
                    &self,
                    $context: &mut StatementContext<'state, 'context, 'block>,
                    $block: BlockRef<'context, 'block>,
                ) -> Option<BlockRef<'context, 'block>> $body
            }
        )+
    };
}

/// The inline-assembly (Yul) counterpart of [`statement_emit`] / [`expression_emit`]:
/// generates `impl EmitYul` for a Yul node. The context is `&mut YulContext` (a Yul
/// `let` declares variables); the output is stated per node because Yul never
/// diverges solx control flow: a statement yields its continuation `BlockRef`
/// (not an `Option`), an expression its `(word, continuation)` pair. The closure
/// binds the node (`|node, context, block|`). Names resolve against the call
/// site's imports.
macro_rules! yul_emit {
    ($node:ty => $output:ty ; |$bound:ident, $context:ident, $block:ident| $body:block) => {
        impl<'context: 'block, 'block> EmitYul<'context, 'block> for $node {
            type Output = $output;

            fn emit<'state>(
                &self,
                $context: &mut YulContext<'state, 'context, 'block>,
                $block: BlockRef<'context, 'block>,
            ) -> Self::Output {
                let $bound = self;
                $body
            }
        }
    };
}
