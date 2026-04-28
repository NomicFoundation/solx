//!
//! Control flow statement lowering: if/else, for, while, do-while.
//!

use melior::ir::BlockRef;
use melior::ir::RegionLike;
use slang_solidity::backend::ir::ast::ForStatementCondition;
use slang_solidity::backend::ir::ast::ForStatementInitialization;
use slang_solidity::backend::ir::ast::Statement;

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
        if_statement: &slang_solidity::backend::ir::ast::IfStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let condition_expression = if_statement.condition();
        let emitter = ExpressionEmitter::new(
            &self.semantic,
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
        let (condition_value, block) = emitter.emit_value(&condition_expression, block)?;
        let condition_boolean = emitter.emit_is_nonzero(condition_value, &block);

        let (then_block, else_block) = self.state.builder.emit_sol_if(condition_boolean, &block);

        // Get the inner regions for creating blocks in the right scope.
        let then_region = solx_mlir::ffi::block_parent_region(&then_block);
        let else_region = solx_mlir::ffi::block_parent_region(&else_block);

        // Emit then body.
        let saved_region = self.region_pointer;
        self.set_region(&then_region);
        let then_end = self.emit(&if_statement.body(), then_block)?;
        if let Some(then_end) = then_end {
            self.state.builder.emit_sol_yield(&then_end);
        } else {
            self.emit_dead_yield(&then_region);
        }

        // Emit else body (or empty yield).
        if let Some(ref else_statement) = if_statement.else_branch() {
            self.set_region(&else_region);
            let else_end = self.emit(else_statement, else_block)?;
            if let Some(else_end) = else_end {
                self.state.builder.emit_sol_yield(&else_end);
            } else {
                self.emit_dead_yield(&else_region);
            }
            self.region_pointer = saved_region;
        } else {
            self.state.builder.emit_sol_yield(&else_block);
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
        for_statement: &slang_solidity::backend::ir::ast::ForStatement,
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
            ForStatementInitialization::TupleDeconstructionStatement(_) => {
                anyhow::bail!("tuple deconstruction in for-init not yet supported")
            }
            ForStatementInitialization::Semicolon => block,
        };

        let (condition_block, body_block, step_block) = self.state.builder.emit_sol_for(&block);
        let body_region = solx_mlir::ffi::block_parent_region(&body_block);
        let saved_region = self.region_pointer;

        // Condition region.
        match for_statement.condition() {
            ForStatementCondition::ExpressionStatement(expression_statement) => {
                let expression = expression_statement.expression();
                let emitter = ExpressionEmitter::new(
                    &self.semantic,
                    self.state,
                    self.environment,
                    self.storage_layout,
                    self.checked,
                );
                let (condition_value, condition_end) =
                    emitter.emit_value(&expression, condition_block)?;
                let condition_boolean = emitter.emit_is_nonzero(condition_value, &condition_end);
                self.state
                    .builder
                    .emit_sol_condition(condition_boolean, &condition_end);
            }
            ForStatementCondition::Semicolon => {
                let true_value = self.state.builder.emit_bool(true, &condition_block);
                self.state
                    .builder
                    .emit_sol_condition(true_value, &condition_block);
            }
        }

        // Body region.
        self.set_region(&body_region);
        let body_end = self.emit(&for_statement.body(), body_block)?;
        if let Some(body_end) = body_end {
            self.state.builder.emit_sol_yield(&body_end);
        }

        // Step region — always unchecked (matches solc: loop step i++ uses sol.add).
        if let Some(ref iterator_expression) = for_statement.iterator() {
            let emitter = ExpressionEmitter::new(
                &self.semantic,
                self.state,
                self.environment,
                self.storage_layout,
                false, // unchecked
            );
            let (_, step_end) = emitter.emit(iterator_expression, step_block)?;
            self.state.builder.emit_sol_yield(&step_end);
        } else {
            self.state.builder.emit_sol_yield(&step_block);
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
        while_statement: &slang_solidity::backend::ir::ast::WhileStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let (condition_block, body_block) = self.state.builder.emit_sol_while(&block);
        let body_region = solx_mlir::ffi::block_parent_region(&body_block);
        let saved_region = self.region_pointer;

        // Condition region.
        let condition_expression = while_statement.condition();
        let emitter = ExpressionEmitter::new(
            &self.semantic,
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
        let (condition_value, condition_end) =
            emitter.emit_value(&condition_expression, condition_block)?;
        let condition_boolean = emitter.emit_is_nonzero(condition_value, &condition_end);
        self.state
            .builder
            .emit_sol_condition(condition_boolean, &condition_end);

        // Body region.
        self.set_region(&body_region);
        let body_end = self.emit(&while_statement.body(), body_block)?;
        if let Some(body_end) = body_end {
            self.state.builder.emit_sol_yield(&body_end);
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
        do_while: &slang_solidity::backend::ir::ast::DoWhileStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let (body_block, condition_block) = self.state.builder.emit_sol_do_while(&block);
        let body_region = solx_mlir::ffi::block_parent_region(&body_block);
        let saved_region = self.region_pointer;

        // Body region (executes first).
        self.set_region(&body_region);
        let body_end = self.emit(&do_while.body(), body_block)?;
        if let Some(body_end) = body_end {
            self.state.builder.emit_sol_yield(&body_end);
        }

        // Condition region.
        let condition_expression = do_while.condition();
        let emitter = ExpressionEmitter::new(
            &self.semantic,
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
        let (condition_value, condition_end) =
            emitter.emit_value(&condition_expression, condition_block)?;
        let condition_boolean = emitter.emit_is_nonzero(condition_value, &condition_end);
        self.state
            .builder
            .emit_sol_condition(condition_boolean, &condition_end);

        self.region_pointer = saved_region;
        Ok(Some(block))
    }

    /// Appends a dead block with `sol.yield` to a region whose live block
    /// already terminated (e.g. with `sol.return`). Matches the solc pattern
    /// where each `sol.if` region always ends with a `sol.yield` block.
    fn emit_dead_yield(&self, region: &melior::ir::Region<'context>) {
        let dead_block = melior::ir::Block::new(&[]);
        self.state.builder.emit_sol_yield(&dead_block);
        region.append_block(dead_block);
    }
}
