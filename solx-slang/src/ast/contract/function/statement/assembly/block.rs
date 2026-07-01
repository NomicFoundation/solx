//!
//! Yul block emission: function-definition hoisting and the statement walk.
//!

use std::collections::HashSet;

use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::RegionLike;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::YulBlock;
use slang_solidity_v2::ast::YulStatement;

use solx_mlir::ods::yul::YieldOperation;

use crate::ast::contract::function::statement::assembly::YulContext;
use crate::ast::emit::emit_yul::EmitYul;

yul_emit!(YulBlock => Option<BlockRef<'context, 'block>>; |yul_block, context, block| {
    let saved_functions: HashSet<NodeId> = context.yul_functions.keys().copied().collect();
    for statement in yul_block.statements().iter() {
        if let YulStatement::YulFunctionDefinition(definition) = &statement {
            context
                .yul_functions
                .insert(definition.node_id(), definition.clone());
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

    let added: Vec<NodeId> = context
        .yul_functions
        .keys()
        .filter(|key| !saved_functions.contains(*key))
        .copied()
        .collect();
    for key in added {
        context.yul_functions.remove(&key);
    }

    if diverged { None } else { Some(current) }
});

/// Emits a Yul block as a control-flow region body (an `if` branch, `for` body, `switch` case),
/// closing with `yul.yield` (in a fresh trailing block if the body already terminated).
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
        let end = match self.emit(context, target_block) {
            Some(end) => end,
            None => region.append_block(Block::new(&[])),
        };
        mlir_op_void!(context.state, &end, YieldOperation.operands(&[]));
    }
}
