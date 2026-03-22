//!
//! Control flow statement lowering: if/else, for, while, do-while.
//!

use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::RegionLike;

use slang_solidity::backend::ir::ast::ForStatementCondition;
use slang_solidity::backend::ir::ast::ForStatementInitialization;
use slang_solidity::backend::ir::ast::Statement;
use solx_mlir::LoopTarget;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Emits an if/else statement with conditional branching.
    pub fn emit_if(
        &mut self,
        if_statement: &slang_solidity::backend::ir::ast::IfStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let condition_expression = if_statement.condition();
        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.region,
            self.storage_layout,
        );
        let (condition_value, block) = emitter.emit(&condition_expression, block)?;
        let condition_boolean = emitter.emit_is_nonzero(condition_value, &block);

        let then_block = self.region.append_block(Block::new(&[]));
        let merge_block = self.region.append_block(Block::new(&[]));

        if let Some(ref else_statement) = if_statement.else_branch() {
            let else_block = self.region.append_block(Block::new(&[]));
            block.append_operation(self.state.builder().llvm_cond_br(
                condition_boolean,
                &then_block,
                &else_block,
                &[],
                &[],
            ));

            let body = if_statement.body();
            let then_end = self.emit(&body, then_block)?;
            if let Some(then_end) = then_end {
                then_end.append_operation(self.state.builder().llvm_br(&merge_block, &[]));
            }

            let else_end = self.emit(else_statement, else_block)?;
            if let Some(else_end) = else_end {
                else_end.append_operation(self.state.builder().llvm_br(&merge_block, &[]));
            }

            if then_end.is_some() || else_end.is_some() {
                Ok(Some(merge_block))
            } else {
                // Both branches terminated — merge_block is unreachable
                // but already in the region; add a terminator to satisfy
                // the MLIR verifier.
                merge_block
                    .append_operation(melior::dialect::llvm::unreachable(self.state.location()));
                Ok(None)
            }
        } else {
            block.append_operation(self.state.builder().llvm_cond_br(
                condition_boolean,
                &then_block,
                &merge_block,
                &[],
                &[],
            ));

            let body = if_statement.body();
            let then_end = self.emit(&body, then_block)?;
            if let Some(then_end) = then_end {
                then_end.append_operation(self.state.builder().llvm_br(&merge_block, &[]));
            }

            Ok(Some(merge_block))
        }
    }

    /// Emits a for loop.
    pub fn emit_for(
        &mut self,
        for_statement: &slang_solidity::backend::ir::ast::ForStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        self.environment.enter_scope();

        // Emit initialization.
        let block = match for_statement.initialization() {
            ForStatementInitialization::VariableDeclarationStatement(declaration) => match self
                .emit(
                    &Statement::VariableDeclarationStatement(declaration.clone()),
                    block,
                )? {
                Some(b) => b,
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
                Some(b) => b,
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

        let condition_block = self.region.append_block(Block::new(&[]));
        let body_block = self.region.append_block(Block::new(&[]));
        let iterator_block = self.region.append_block(Block::new(&[]));
        let exit_block = self.region.append_block(Block::new(&[]));

        block.append_operation(self.state.builder().llvm_br(&condition_block, &[]));

        // Condition.
        match for_statement.condition() {
            ForStatementCondition::ExpressionStatement(expression_statement) => {
                let expression = expression_statement.expression();
                let emitter = ExpressionEmitter::new(
                    self.state,
                    self.environment,
                    self.region,
                    self.storage_layout,
                );
                let (condition_value, condition_end) =
                    emitter.emit(&expression, condition_block)?;
                let condition_boolean = emitter.emit_is_nonzero(condition_value, &condition_end);
                condition_end.append_operation(self.state.builder().llvm_cond_br(
                    condition_boolean,
                    &body_block,
                    &exit_block,
                    &[],
                    &[],
                ));
            }
            ForStatementCondition::Semicolon => {
                condition_block.append_operation(self.state.builder().llvm_br(&body_block, &[]));
            }
        }

        // Body with loop targets.
        self.environment
            .push_loop(LoopTarget::new(exit_block, iterator_block));
        let body = for_statement.body();
        let body_end = self.emit(&body, body_block)?;
        self.environment.pop_loop();

        if let Some(body_end) = body_end {
            body_end.append_operation(self.state.builder().llvm_br(&iterator_block, &[]));
        }

        // Iterator.
        if let Some(ref iterator_expression) = for_statement.iterator() {
            let expression_emitter = ExpressionEmitter::new(
                self.state,
                self.environment,
                self.region,
                self.storage_layout,
            );
            let (_value, iterator_end) =
                expression_emitter.emit(iterator_expression, iterator_block)?;
            iterator_end.append_operation(self.state.builder().llvm_br(&condition_block, &[]));
        } else {
            iterator_block.append_operation(self.state.builder().llvm_br(&condition_block, &[]));
        }

        self.environment.exit_scope();
        Ok(Some(exit_block))
    }

    /// Emits a while loop.
    pub fn emit_while(
        &mut self,
        while_statement: &slang_solidity::backend::ir::ast::WhileStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let condition_block = self.region.append_block(Block::new(&[]));
        let body_block = self.region.append_block(Block::new(&[]));
        let exit_block = self.region.append_block(Block::new(&[]));

        block.append_operation(self.state.builder().llvm_br(&condition_block, &[]));

        let condition_expression = while_statement.condition();
        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.region,
            self.storage_layout,
        );
        let (condition_value, condition_end) =
            emitter.emit(&condition_expression, condition_block)?;
        let condition_boolean = emitter.emit_is_nonzero(condition_value, &condition_end);
        condition_end.append_operation(self.state.builder().llvm_cond_br(
            condition_boolean,
            &body_block,
            &exit_block,
            &[],
            &[],
        ));

        self.environment
            .push_loop(LoopTarget::new(exit_block, condition_block));
        let body = while_statement.body();
        let body_end = self.emit(&body, body_block)?;
        self.environment.pop_loop();

        if let Some(body_end) = body_end {
            body_end.append_operation(self.state.builder().llvm_br(&condition_block, &[]));
        }

        Ok(Some(exit_block))
    }

    /// Emits a do-while loop.
    pub fn emit_do_while(
        &mut self,
        do_while: &slang_solidity::backend::ir::ast::DoWhileStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let body_block = self.region.append_block(Block::new(&[]));
        let condition_block = self.region.append_block(Block::new(&[]));
        let exit_block = self.region.append_block(Block::new(&[]));

        block.append_operation(self.state.builder().llvm_br(&body_block, &[]));

        self.environment
            .push_loop(LoopTarget::new(exit_block, condition_block));
        let body = do_while.body();
        let body_end = self.emit(&body, body_block)?;
        self.environment.pop_loop();

        if let Some(body_end) = body_end {
            body_end.append_operation(self.state.builder().llvm_br(&condition_block, &[]));
        }

        let condition_expression = do_while.condition();
        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.region,
            self.storage_layout,
        );
        let (condition_value, condition_end) =
            emitter.emit(&condition_expression, condition_block)?;
        let condition_boolean = emitter.emit_is_nonzero(condition_value, &condition_end);
        condition_end.append_operation(self.state.builder().llvm_cond_br(
            condition_boolean,
            &body_block,
            &exit_block,
            &[],
            &[],
        ));

        Ok(Some(exit_block))
    }
}
