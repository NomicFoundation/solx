//!
//! Yul block emission: function-definition hoisting and the statement walk.
//!

use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::RegionLike;
use slang_solidity_v2::ast::YulBlock;
use slang_solidity_v2::ast::YulStatement;
use solx_mlir::ods::yul::*;

use crate::ast::Emit;
use crate::ast::contract::function::statement::assembly::YulContext;

// Yul resolves function calls regardless of textual order, so a block first
// pre-registers its `function` definitions, then emits each non-definition
// statement (`None` once a `break`/`continue` diverges), then drops the
// definitions added here so an enclosing scope's stay intact. The lexical scope
// is the caller's: the top-level `assembly` block reuses the function scope, a
// nested `{ … }` brackets its own.
yul_emit!(YulBlock => Option<BlockRef<'context, 'block>>; |yul_block, context, block| {
    let saved_functions: Vec<String> = context.yul_functions.keys().cloned().collect();
    for statement in yul_block.statements().iter() {
        if let YulStatement::YulFunctionDefinition(definition) = &statement {
            context
                .yul_functions
                .insert(definition.name().name(), definition.clone());
        }
    }

    let mut current = block;
    let mut diverged = false;
    for statement in yul_block.statements().iter() {
        if matches!(statement, YulStatement::YulFunctionDefinition(_)) {
            continue;
        }
        match statement.emit(context, current) {
            Some(next) => current = next,
            None => {
                diverged = true;
                break;
            }
        }
    }

    let added: Vec<String> = context
        .yul_functions
        .keys()
        .filter(|key| !saved_functions.contains(*key))
        .cloned()
        .collect();
    for key in added {
        context.yul_functions.remove(&key);
    }

    if diverged { None } else { Some(current) }
});

/// Emits a Yul block as a control-flow region body — an `if` branch, a `for`
/// body / step, a `switch` case / default. Switches into the region, walks the
/// statements, and closes with `yul.yield`; a body that already terminated with
/// `break`/`continue` puts the required `yul.yield` in a fresh trailing block,
/// matching solc.
pub trait EmitRegionBody<'context, 'block, 'state, 'scope> {
    /// Emits this block into the region owning `target_block`.
    fn emit_region_body(
        &self,
        context: &'scope mut YulContext<'state, 'context, 'block>,
        target_block: BlockRef<'context, 'block>,
    );
}

impl<'state, 'context, 'block, 'scope> EmitRegionBody<'context, 'block, 'state, 'scope> for YulBlock
where
    'context: 'block,
    'context: 'state,
    'block: 'state,
    'state: 'scope,
{
    fn emit_region_body(
        &self,
        context: &'scope mut YulContext<'state, 'context, 'block>,
        target_block: BlockRef<'context, 'block>,
    ) {
        let region = target_block
            .parent_region()
            .expect("region body block has a parent region");
        let saved_region = context.region_pointer;
        context.set_region(&region);
        let end = match self.emit(context, target_block) {
            Some(end) => end,
            None => region.append_block(Block::new(&[])),
        };
        mlir_op_void!(&context.state.builder, &end, YieldOperation.operands(&[]));
        context.region_pointer = saved_region;
    }
}
