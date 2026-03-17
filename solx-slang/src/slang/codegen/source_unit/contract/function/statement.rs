//!
//! Statement lowering to MLIR operations.
//!

use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::RegionLike;

use slang_solidity::backend::ir::ast::ElementaryType;
use slang_solidity::backend::ir::ast::ForStatementCondition;
use slang_solidity::backend::ir::ast::ForStatementInitialization;
use slang_solidity::backend::ir::ast::Statement;
use slang_solidity::backend::ir::ast::Statements;
use slang_solidity::backend::ir::ast::TypeName;
use solx_mlir::Environment;
use solx_mlir::LoopTarget;

use crate::slang::codegen::MlirContext;

use crate::slang::codegen::source_unit::contract::function::expression::ExpressionEmitter;

/// Lowers Solidity statements to MLIR operations with control flow.
///
/// Returns `Some(block)` as the continuation block, or `None` when control
/// flow has been terminated (by `return`, `break`, or `continue`).
pub(crate) struct StatementEmitter<'state, 'context, 'block> {
    /// The shared MLIR context.
    state: &'state MlirContext<'context>,
    /// Variable environment (mutable for new declarations and loop targets).
    environment: &'state mut Environment<'context, 'block>,
    /// The function region for creating new blocks.
    region: &'state Region<'context>,
}

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Creates a new statement emitter.
    pub(crate) fn new(
        state: &'state MlirContext<'context>,
        environment: &'state mut Environment<'context, 'block>,
        region: &'state Region<'context>,
    ) -> Self {
        Self {
            state,
            environment,
            region,
        }
    }

    /// Emits MLIR for a statement.
    ///
    /// Returns `Some(block)` as the continuation block for the next statement,
    /// or `None` if control flow was terminated.
    pub(crate) fn emit(
        &mut self,
        statement: &Statement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        match statement {
            Statement::VariableDeclarationStatement(declaration) => {
                self.emit_variable_declaration(declaration, block)
            }
            Statement::ExpressionStatement(expression_statement) => {
                let expr = expression_statement.expression();
                let emitter = ExpressionEmitter::new(self.state, self.environment, self.region);
                let (_value, block) = emitter.emit(&expr, block)?;
                Ok(Some(block))
            }
            Statement::ReturnStatement(return_statement) => {
                self.emit_return(return_statement, block)
            }
            Statement::IfStatement(if_statement) => self.emit_if(if_statement, block),
            Statement::ForStatement(for_statement) => self.emit_for(for_statement, block),
            Statement::WhileStatement(while_statement) => self.emit_while(while_statement, block),
            Statement::DoWhileStatement(do_while) => self.emit_do_while(do_while, block),
            Statement::BreakStatement(_) => self.emit_break(block),
            Statement::ContinueStatement(_) => self.emit_continue(block),
            Statement::Block(inner) => self.emit_block(inner.statements(), block),
            Statement::UncheckedBlock(inner) => self.emit_block(inner.block().statements(), block),
            _ => anyhow::bail!(
                "unsupported statement: {:?}",
                std::mem::discriminant(statement)
            ),
        }
    }

    /// Emits a sequence of statements.
    fn emit_block(
        &mut self,
        statements: Statements,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let mut current = block;
        for statement in statements.iter() {
            match self.emit(&statement, current)? {
                Some(next) => current = next,
                None => return Ok(None),
            }
        }
        Ok(Some(current))
    }

    /// Emits a variable declaration with optional initializer.
    fn emit_variable_declaration(
        &mut self,
        declaration: &slang_solidity::backend::ir::ast::VariableDeclarationStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let name = declaration.name().name();
        let is_signed = declaration.type_name().is_some_and(|ref type_name| {
            matches!(
                type_name,
                TypeName::ElementaryType(ElementaryType::IntKeyword(_))
            )
        });

        let emitter = ExpressionEmitter::new(self.state, self.environment, self.region);
        let pointer = emitter.emit_alloca(&block);

        let block = if let Some(ref initializer_expression) = declaration.value() {
            let (initial_value, block) = emitter.emit(initializer_expression, block)?;
            emitter.emit_store(initial_value, pointer, &block);
            block
        } else {
            let zero = emitter.emit_i256_constant(0, &block);
            emitter.emit_store(zero, pointer, &block);
            block
        };

        if is_signed {
            self.environment.mark_signed(&name);
        }
        self.environment.define_variable(name, pointer);
        Ok(Some(block))
    }

    /// Emits a return statement.
    fn emit_return(
        &mut self,
        return_statement: &slang_solidity::backend::ir::ast::ReturnStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        if let Some(ref expr) = return_statement.expression() {
            let emitter = ExpressionEmitter::new(self.state, self.environment, self.region);
            let (value, block) = emitter.emit(expr, block)?;
            self.state.emit_sol_return(&[value], &block);
        } else {
            self.state.emit_sol_return(&[], &block);
        }

        Ok(None)
    }

    /// Emits an if/else statement with conditional branching.
    fn emit_if(
        &mut self,
        if_statement: &slang_solidity::backend::ir::ast::IfStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let condition_expr = if_statement.condition();
        let emitter = ExpressionEmitter::new(self.state, self.environment, self.region);
        let (condition_value, block) = emitter.emit(&condition_expr, block)?;
        let condition_boolean = emitter.emit_is_nonzero(condition_value, &block);

        let then_block = self.region.append_block(Block::new(&[]));
        let merge_block = self.region.append_block(Block::new(&[]));

        if let Some(ref else_statement) = if_statement.else_branch() {
            let else_block = self.region.append_block(Block::new(&[]));
            block.append_operation(self.state.llvm_cond_br(
                condition_boolean,
                &then_block,
                &else_block,
                &[],
                &[],
            ));

            let body = if_statement.body();
            let then_end = self.emit(&body, then_block)?;
            if let Some(then_end) = then_end {
                then_end.append_operation(self.state.llvm_br(&merge_block, &[]));
            }

            let else_end = self.emit(else_statement, else_block)?;
            if let Some(else_end) = else_end {
                else_end.append_operation(self.state.llvm_br(&merge_block, &[]));
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
            block.append_operation(self.state.llvm_cond_br(
                condition_boolean,
                &then_block,
                &merge_block,
                &[],
                &[],
            ));

            let body = if_statement.body();
            let then_end = self.emit(&body, then_block)?;
            if let Some(then_end) = then_end {
                then_end.append_operation(self.state.llvm_br(&merge_block, &[]));
            }

            Ok(Some(merge_block))
        }
    }

    /// Emits a for loop.
    fn emit_for(
        &mut self,
        for_statement: &slang_solidity::backend::ir::ast::ForStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        // Emit initialization.
        let block = match for_statement.initialization() {
            ForStatementInitialization::VariableDeclarationStatement(declaration) => match self
                .emit(
                    &Statement::VariableDeclarationStatement(declaration.clone()),
                    block,
                )? {
                Some(b) => b,
                None => return Ok(None),
            },
            ForStatementInitialization::ExpressionStatement(expression_statement) => match self
                .emit(
                    &Statement::ExpressionStatement(expression_statement.clone()),
                    block,
                )? {
                Some(b) => b,
                None => return Ok(None),
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

        block.append_operation(self.state.llvm_br(&condition_block, &[]));

        // Condition.
        match for_statement.condition() {
            ForStatementCondition::ExpressionStatement(expression_statement) => {
                let expr = expression_statement.expression();
                let emitter = ExpressionEmitter::new(self.state, self.environment, self.region);
                let (condition_value, condition_end) = emitter.emit(&expr, condition_block)?;
                let condition_boolean = emitter.emit_is_nonzero(condition_value, &condition_end);
                condition_end.append_operation(self.state.llvm_cond_br(
                    condition_boolean,
                    &body_block,
                    &exit_block,
                    &[],
                    &[],
                ));
            }
            ForStatementCondition::Semicolon => {
                condition_block.append_operation(self.state.llvm_br(&body_block, &[]));
            }
        }

        // Body with loop targets.
        self.environment
            .push_loop(LoopTarget::new(exit_block, iterator_block));
        let body = for_statement.body();
        let body_end = self.emit(&body, body_block)?;
        self.environment.pop_loop();

        if let Some(body_end) = body_end {
            body_end.append_operation(self.state.llvm_br(&iterator_block, &[]));
        }

        // Iterator.
        if let Some(ref iterator_expression) = for_statement.iterator() {
            let expression_emitter =
                ExpressionEmitter::new(self.state, self.environment, self.region);
            let (_value, iterator_end) =
                expression_emitter.emit(iterator_expression, iterator_block)?;
            iterator_end.append_operation(self.state.llvm_br(&condition_block, &[]));
        } else {
            iterator_block.append_operation(self.state.llvm_br(&condition_block, &[]));
        }

        Ok(Some(exit_block))
    }

    /// Emits a while loop.
    fn emit_while(
        &mut self,
        while_statement: &slang_solidity::backend::ir::ast::WhileStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let condition_block = self.region.append_block(Block::new(&[]));
        let body_block = self.region.append_block(Block::new(&[]));
        let exit_block = self.region.append_block(Block::new(&[]));

        block.append_operation(self.state.llvm_br(&condition_block, &[]));

        let condition_expr = while_statement.condition();
        let emitter = ExpressionEmitter::new(self.state, self.environment, self.region);
        let (condition_value, condition_end) = emitter.emit(&condition_expr, condition_block)?;
        let condition_boolean = emitter.emit_is_nonzero(condition_value, &condition_end);
        condition_end.append_operation(self.state.llvm_cond_br(
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
            body_end.append_operation(self.state.llvm_br(&condition_block, &[]));
        }

        Ok(Some(exit_block))
    }

    /// Emits a do-while loop.
    fn emit_do_while(
        &mut self,
        do_while: &slang_solidity::backend::ir::ast::DoWhileStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let body_block = self.region.append_block(Block::new(&[]));
        let condition_block = self.region.append_block(Block::new(&[]));
        let exit_block = self.region.append_block(Block::new(&[]));

        block.append_operation(self.state.llvm_br(&body_block, &[]));

        self.environment
            .push_loop(LoopTarget::new(exit_block, condition_block));
        let body = do_while.body();
        let body_end = self.emit(&body, body_block)?;
        self.environment.pop_loop();

        if let Some(body_end) = body_end {
            body_end.append_operation(self.state.llvm_br(&condition_block, &[]));
        }

        let condition_expr = do_while.condition();
        let emitter = ExpressionEmitter::new(self.state, self.environment, self.region);
        let (condition_value, condition_end) = emitter.emit(&condition_expr, condition_block)?;
        let condition_boolean = emitter.emit_is_nonzero(condition_value, &condition_end);
        condition_end.append_operation(self.state.llvm_cond_br(
            condition_boolean,
            &body_block,
            &exit_block,
            &[],
            &[],
        ));

        Ok(Some(exit_block))
    }

    /// Emits a break statement.
    fn emit_break(
        &self,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let target = self
            .environment
            .current_loop()
            .ok_or_else(|| anyhow::anyhow!("break outside of loop"))?;
        block.append_operation(self.state.llvm_br(&target.break_block(), &[]));
        Ok(None)
    }

    /// Emits a continue statement.
    fn emit_continue(
        &self,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let target = self
            .environment
            .current_loop()
            .ok_or_else(|| anyhow::anyhow!("continue outside of loop"))?;
        block.append_operation(self.state.llvm_br(&target.continue_block(), &[]));
        Ok(None)
    }
}
