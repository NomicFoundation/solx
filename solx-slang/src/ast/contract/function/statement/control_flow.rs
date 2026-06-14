//!
//! Control flow statement lowering: if/else, for, while, do-while.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::RegionLike;
use slang_solidity_v2::ast::DoWhileStatement;
use slang_solidity_v2::ast::ForStatement;
use slang_solidity_v2::ast::ForStatementCondition;
use slang_solidity_v2::ast::ForStatementInitialization;
use slang_solidity_v2::ast::IfStatement;
use slang_solidity_v2::ast::Statement;
use slang_solidity_v2::ast::WhileStatement;
use solx_mlir::ods::sol::ConditionOperation;
use solx_mlir::ods::sol::YieldOperation;

use super::discarded::Discarded;
use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::function::statement::StatementContext;

impl<'state, 'context, 'block> StatementContext<'state, 'context, 'block> {
    /// Emits an if/else statement using `sol.if`.
    ///
    /// # Errors
    ///
    /// Returns an error if the condition or body contains unsupported constructs.
    pub fn emit_if(
        &mut self,
        if_statement: &slang_solidity_v2::ast::IfStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let condition_expression = if_statement.condition();
        let emitter = ExpressionContext::from(&*self);
        let BlockAnd {
            value: condition_value,
            block,
        } = condition_expression.emit(&emitter, block)?;
        let condition_boolean = condition_value
            .is_nonzero(&self.state.builder, &block)
            .into_mlir();

        let (then_block, else_block) = self.state.builder.emit_sol_if(condition_boolean, &block);

        // Get the inner regions for creating blocks in the right scope.
        let then_region = solx_mlir::ffi::block_parent_region(&then_block);
        let else_region = solx_mlir::ffi::block_parent_region(&else_block);

        // Emit then body.
        let saved_region = self.region_pointer;
        self.set_region(&then_region);
        let then_end = if_statement.body().emit(self, then_block)?;
        if let Some(then_end) = then_end {
            sol_op_void!(&self.state.builder, &then_end, YieldOperation.ins(&[]));
        } else {
            self.emit_dead_yield(&then_region);
        }

        // Emit else body (or empty yield).
        if let Some(ref else_statement) = if_statement.else_branch() {
            self.set_region(&else_region);
            let else_end = else_statement.emit(self, else_block)?;
            if let Some(else_end) = else_end {
                sol_op_void!(&self.state.builder, &else_end, YieldOperation.ins(&[]));
            } else {
                self.emit_dead_yield(&else_region);
            }
            self.region_pointer = saved_region;
        } else {
            sol_op_void!(&self.state.builder, &else_block, YieldOperation.ins(&[]));
            self.region_pointer = saved_region;
        }

        Ok(Some(block))
    }

    /// Emits a for loop using `sol.for`.
    ///
    /// # Errors
    ///
    /// Returns an error if the initialization, condition, body, or step
    /// contains unsupported constructs.
    pub fn emit_for(
        &mut self,
        for_statement: &slang_solidity_v2::ast::ForStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        self.environment.enter_scope();

        // Emit initialization in the current block.
        let block = match for_statement.initialization() {
            ForStatementInitialization::VariableDeclarationStatement(declaration) => {
                let statement = Statement::VariableDeclarationStatement(declaration.clone());
                match statement.emit(self, block)? {
                    Some(block) => block,
                    None => {
                        self.environment.exit_scope();
                        return Ok(None);
                    }
                }
            }
            ForStatementInitialization::ExpressionStatement(expression_statement) => {
                let statement = Statement::ExpressionStatement(expression_statement.clone());
                match statement.emit(self, block)? {
                    Some(block) => block,
                    None => {
                        self.environment.exit_scope();
                        return Ok(None);
                    }
                }
            }
            ForStatementInitialization::Semicolon(_) => block,
        };

        let (condition_block, body_block, step_block) = self.state.builder.emit_sol_for(&block);
        let body_region = solx_mlir::ffi::block_parent_region(&body_block);
        let saved_region = self.region_pointer;

        // Condition region.
        match for_statement.condition() {
            ForStatementCondition::ExpressionStatement(expression_statement) => {
                self.emit_loop_condition(&expression_statement.expression(), condition_block)?;
            }
            ForStatementCondition::Semicolon(_) => {
                let true_value =
                    crate::ast::Value::boolean(true, &self.state.builder, &condition_block)
                        .into_mlir();
                sol_op_void!(
                    &self.state.builder,
                    &condition_block,
                    ConditionOperation.condition(true_value)
                );
            }
        }

        // Body region.
        self.set_region(&body_region);
        let body_end = for_statement.body().emit(self, body_block)?;
        if let Some(body_end) = body_end {
            sol_op_void!(&self.state.builder, &body_end, YieldOperation.ins(&[]));
        }

        // Step region — always unchecked (matches solc: loop step i++ uses sol.add).
        if let Some(ref iterator_expression) = for_statement.iterator() {
            let emitter = ExpressionContext::new(
                self.state,
                self.environment,
                self.storage_layout,
                // The loop step is always unchecked; solc emits `sol.add` for `i++`.
                ArithmeticMode::Unchecked,
            );
            // The step is in statement position: its value is discarded and it
            // may be a value-less producer (a void call or `delete`).
            let step_end = Discarded(iterator_expression).emit(&emitter, step_block)?;
            sol_op_void!(&self.state.builder, &step_end, YieldOperation.ins(&[]));
        } else {
            sol_op_void!(&self.state.builder, &step_block, YieldOperation.ins(&[]));
        }

        self.region_pointer = saved_region;
        self.environment.exit_scope();
        Ok(Some(block))
    }

    /// Emits a while loop using `sol.while`.
    ///
    /// # Errors
    ///
    /// Returns an error if the condition or body contains unsupported constructs.
    pub fn emit_while(
        &mut self,
        while_statement: &slang_solidity_v2::ast::WhileStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let (condition_block, body_block) = self.state.builder.emit_sol_while(&block);
        let body_region = solx_mlir::ffi::block_parent_region(&body_block);
        let saved_region = self.region_pointer;

        // Condition region.
        self.emit_loop_condition(&while_statement.condition(), condition_block)?;

        // Body region.
        self.set_region(&body_region);
        let body_end = while_statement.body().emit(self, body_block)?;
        if let Some(body_end) = body_end {
            sol_op_void!(&self.state.builder, &body_end, YieldOperation.ins(&[]));
        }

        self.region_pointer = saved_region;
        Ok(Some(block))
    }

    /// Emits a do-while loop using `sol.do`.
    ///
    /// # Errors
    ///
    /// Returns an error if the body or condition contains unsupported constructs.
    pub fn emit_do_while(
        &mut self,
        do_while: &slang_solidity_v2::ast::DoWhileStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let (body_block, condition_block) = self.state.builder.emit_sol_do_while(&block);
        let body_region = solx_mlir::ffi::block_parent_region(&body_block);
        let saved_region = self.region_pointer;

        // Body region (executes first).
        self.set_region(&body_region);
        let body_end = do_while.body().emit(self, body_block)?;
        if let Some(body_end) = body_end {
            sol_op_void!(&self.state.builder, &body_end, YieldOperation.ins(&[]));
        }

        // Condition region.
        self.emit_loop_condition(&do_while.condition(), condition_block)?;

        self.region_pointer = saved_region;
        Ok(Some(block))
    }

    /// Appends a dead block with `sol.yield` to a region whose live block
    /// already terminated (e.g. with `sol.return`). Matches the solc pattern
    /// where each `sol.if` region always ends with a `sol.yield` block.
    fn emit_dead_yield(&self, region: &melior::ir::Region<'context>) {
        let dead_block = melior::ir::Block::new(&[]);
        sol_op_void!(&self.state.builder, &dead_block, YieldOperation.ins(&[]));
        region.append_block(dead_block);
    }
}

statement_emit!(IfStatement; |node, context, block| { context.emit_if(node, block) });

statement_emit!(ForStatement; |node, context, block| { context.emit_for(node, block) });

statement_emit!(WhileStatement; |node, context, block| { context.emit_while(node, block) });

statement_emit!(DoWhileStatement; |node, context, block| {
    context.emit_do_while(node, block)
});
