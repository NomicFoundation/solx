//!
//! Control-flow ops on the [`Effect`] entity: block terminators and region-bearing constructs.
//!
//! Terminators (`sol.return`/`break`/`continue`/`yield`/`condition`) close the current block.
//! Region-bearing ops (`sol.if`/`for`/`while`/`do_while`) open fresh regions and hand back their
//! entry blocks for the caller to emit into and terminate.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;

use crate::Effect;
use crate::Value;
use crate::ods::sol::BreakOperation;
use crate::ods::sol::ConditionOperation;
use crate::ods::sol::ContinueOperation;
use crate::ods::sol::DoWhileOperation;
use crate::ods::sol::ForOperation;
use crate::ods::sol::IfOperation;
use crate::ods::sol::ReturnOperation;
use crate::ods::sol::WhileOperation;
use crate::ods::sol::YieldOperation;

impl<'a, 'context, 'block> Effect<'a, 'context, 'block> {
    /// Emits `sol.return` carrying `operands`.
    pub fn r#return(self, operands: &[Value<'context, 'block>]) {
        let operands = operands
            .iter()
            .map(|operand| operand.into_mlir())
            .collect::<Vec<_>>();
        mlir_op_void!(
            self.context,
            &self.block,
            ReturnOperation.operands(operands.as_slice())
        );
    }

    /// Emits `sol.break`.
    pub fn r#break(self) {
        mlir_op_void!(self.context, &self.block, BreakOperation);
    }

    /// Emits `sol.continue`.
    pub fn r#continue(self) {
        mlir_op_void!(self.context, &self.block, ContinueOperation);
    }

    /// Emits `sol.yield` carrying `results`, terminating a region body.
    pub fn r#yield(self, results: &[Value<'context, 'block>]) {
        let results = results
            .iter()
            .map(|result| result.into_mlir())
            .collect::<Vec<_>>();
        mlir_op_void!(
            self.context,
            &self.block,
            YieldOperation.ins(results.as_slice())
        );
    }

    /// Emits `sol.condition` gating a loop region on `condition`.
    pub fn condition(self, condition: Value<'context, 'block>) {
        mlir_op_void!(
            self.context,
            &self.block,
            ConditionOperation.condition(condition)
        );
    }

    /// Emits `sol.if` and returns the then-region entry block, plus the else-region entry block
    /// when `with_else`; otherwise the else region is left empty.
    pub fn branch(
        self,
        condition: Value<'context, 'block>,
        with_else: bool,
    ) -> (
        BlockRef<'context, 'block>,
        Option<BlockRef<'context, 'block>>,
    ) {
        mlir_region_op!(self.context, &self.block, IfOperation.cond(condition); then_region; else_region if with_else)
    }

    /// Emits `sol.for` and returns the condition-, body-, and step-region entry blocks.
    pub fn for_loop(
        self,
    ) -> (
        BlockRef<'context, 'block>,
        BlockRef<'context, 'block>,
        BlockRef<'context, 'block>,
    ) {
        mlir_region_op!(self.context, &self.block, ForOperation; cond, body, step)
    }

    /// Emits `sol.while` and returns the condition- and body-region entry blocks.
    pub fn while_loop(self) -> (BlockRef<'context, 'block>, BlockRef<'context, 'block>) {
        mlir_region_op!(self.context, &self.block, WhileOperation; cond, body)
    }

    /// Emits `sol.do_while` and returns the body- and condition-region entry blocks.
    pub fn do_while(self) -> (BlockRef<'context, 'block>, BlockRef<'context, 'block>) {
        mlir_region_op!(self.context, &self.block, DoWhileOperation; body, cond)
    }
}
