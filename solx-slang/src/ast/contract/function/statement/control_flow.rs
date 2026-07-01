//!
//! Control flow statement lowering: if/else, for, while, do-while.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::RegionLike;
use slang_solidity_v2::ast::ForStatementCondition;
use slang_solidity_v2::ast::ForStatementInitialization;
use slang_solidity_v2::ast::Statement;

use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::ConditionOperation;
use solx_mlir::ods::sol::DoWhileOperation;
use solx_mlir::ods::sol::ForOperation;
use solx_mlir::ods::sol::IfOperation;
use solx_mlir::ods::sol::WhileOperation;
use solx_mlir::ods::sol::YieldOperation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
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
        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
        let (condition_value, block) = emitter.emit_value(&condition_expression, block)?;
        let condition_boolean = emitter.emit_is_nonzero(condition_value, &block);

        let (then_block, else_block) = mlir_region_op!(
            self.state, &block,
            IfOperation.cond(condition_boolean); then_region, else_region
        );

        let then_region = solx_mlir::ffi::block_parent_region(&then_block);
        let else_region = solx_mlir::ffi::block_parent_region(&else_block);

        let saved_region = self.region_pointer;
        self.set_region(&then_region);
        let then_end = self.emit(&if_statement.body(), then_block)?;
        if let Some(then_end) = then_end {
            mlir_op_void!(self.state, &then_end, YieldOperation.ins(&[]));
        } else {
            self.emit_dead_yield(&then_region);
        }

        if let Some(ref else_statement) = if_statement.else_branch() {
            self.set_region(&else_region);
            let else_end = self.emit(else_statement, else_block)?;
            if let Some(else_end) = else_end {
                mlir_op_void!(self.state, &else_end, YieldOperation.ins(&[]));
            } else {
                self.emit_dead_yield(&else_region);
            }
            self.region_pointer = saved_region;
        } else {
            mlir_op_void!(self.state, &else_block, YieldOperation.ins(&[]));
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
            ForStatementInitialization::VariableDeclarationStatement(declaration) => match self
                .emit(
                    &Statement::VariableDeclarationStatement(declaration.clone()),
                    block,
                )? {
                Some(block) => block,
                None => {
                    self.environment.exit_scope();
                    return Ok(None);
                }
            },
            ForStatementInitialization::ExpressionStatement(expression_statement) => match self
                .emit(
                    &Statement::ExpressionStatement(expression_statement.clone()),
                    block,
                )? {
                Some(block) => block,
                None => {
                    self.environment.exit_scope();
                    return Ok(None);
                }
            },
            ForStatementInitialization::Semicolon(_) => block,
        };

        let (condition_block, body_block, step_block) =
            mlir_region_op!(self.state, &block, ForOperation; cond, body, step);
        let body_region = solx_mlir::ffi::block_parent_region(&body_block);
        let saved_region = self.region_pointer;

        match for_statement.condition() {
            ForStatementCondition::ExpressionStatement(expression_statement) => {
                let expression = expression_statement.expression();
                let emitter = ExpressionEmitter::new(
                    self.state,
                    self.environment,
                    self.storage_layout,
                    self.checked,
                );
                let (condition_value, condition_end) =
                    emitter.emit_value(&expression, condition_block)?;
                let condition_boolean = emitter.emit_is_nonzero(condition_value, &condition_end);
                mlir_op_void!(
                    self.state,
                    &condition_end,
                    ConditionOperation.condition(condition_boolean)
                );
            }
            ForStatementCondition::Semicolon(_) => {
                let true_value =
                    AstValue::boolean(true, self.state, &condition_block).into_mlir();
                mlir_op_void!(
                    self.state,
                    &condition_block,
                    ConditionOperation.condition(true_value)
                );
            }
        }

        self.set_region(&body_region);
        let body_end = self.emit(&for_statement.body(), body_block)?;
        if let Some(body_end) = body_end {
            mlir_op_void!(self.state, &body_end, YieldOperation.ins(&[]));
        }

        if let Some(ref iterator_expression) = for_statement.iterator() {
            let emitter =
                ExpressionEmitter::new(self.state, self.environment, self.storage_layout, false);
            let (_, step_end) = emitter.emit(iterator_expression, step_block)?;
            mlir_op_void!(self.state, &step_end, YieldOperation.ins(&[]));
        } else {
            mlir_op_void!(self.state, &step_block, YieldOperation.ins(&[]));
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
        let (condition_block, body_block) =
            mlir_region_op!(self.state, &block, WhileOperation; cond, body);
        let body_region = solx_mlir::ffi::block_parent_region(&body_block);
        let saved_region = self.region_pointer;

        let condition_expression = while_statement.condition();
        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
        let (condition_value, condition_end) =
            emitter.emit_value(&condition_expression, condition_block)?;
        let condition_boolean = emitter.emit_is_nonzero(condition_value, &condition_end);
        mlir_op_void!(
            self.state,
            &condition_end,
            ConditionOperation.condition(condition_boolean)
        );

        self.set_region(&body_region);
        let body_end = self.emit(&while_statement.body(), body_block)?;
        if let Some(body_end) = body_end {
            mlir_op_void!(self.state, &body_end, YieldOperation.ins(&[]));
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
        let (body_block, condition_block) =
            mlir_region_op!(self.state, &block, DoWhileOperation; body, cond);
        let body_region = solx_mlir::ffi::block_parent_region(&body_block);
        let saved_region = self.region_pointer;

        self.set_region(&body_region);
        let body_end = self.emit(&do_while.body(), body_block)?;
        if let Some(body_end) = body_end {
            mlir_op_void!(self.state, &body_end, YieldOperation.ins(&[]));
        }

        let condition_expression = do_while.condition();
        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
        let (condition_value, condition_end) =
            emitter.emit_value(&condition_expression, condition_block)?;
        let condition_boolean = emitter.emit_is_nonzero(condition_value, &condition_end);
        mlir_op_void!(
            self.state,
            &condition_end,
            ConditionOperation.condition(condition_boolean)
        );

        self.region_pointer = saved_region;
        Ok(Some(block))
    }

    /// Appends a dead block with `sol.yield` to a region whose live block
    /// already terminated (e.g. with `sol.return`). Matches the solc pattern
    /// where each `sol.if` region always ends with a `sol.yield` block.
    fn emit_dead_yield(&self, region: &melior::ir::Region<'context>) {
        let dead_block = melior::ir::Block::new(&[]);
        mlir_op_void!(self.state, &dead_block, YieldOperation.ins(&[]));
        region.append_block(dead_block);
    }
}
