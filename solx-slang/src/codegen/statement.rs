//!
//! Statement lowering to MLIR operations.
//!

use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::RegionLike;

use slang_solidity::backend::ir::ir2_flat_contracts::ForStatementCondition;
use slang_solidity::backend::ir::ir2_flat_contracts::ForStatementInitialization;
use slang_solidity::backend::ir::ir2_flat_contracts::Statement;

use solx_mlir::Environment;
use solx_mlir::LoopTarget;
use solx_mlir::ops;

use crate::codegen::MlirContext;
use crate::codegen::expression::ExpressionEmitter;
use crate::codegen::types::TypeMapper;

/// Lowers Solidity statements to MLIR operations with control flow.
///
/// Returns `Some(block)` as the continuation block, or `None` when control
/// flow has been terminated (by `return`, `break`, or `continue`).
pub struct StatementEmitter<'a, 'c, 'b> {
    /// The shared MLIR context.
    state: &'a MlirContext<'c>,
    /// Variable environment (mutable for new declarations and loop targets).
    env: &'a mut Environment<'c, 'b>,
    /// The function region for creating new blocks.
    region: &'a Region<'c>,
}

impl<'a, 'c, 'b> StatementEmitter<'a, 'c, 'b> {
    /// Creates a new statement emitter.
    pub fn new(
        state: &'a MlirContext<'c>,
        env: &'a mut Environment<'c, 'b>,
        region: &'a Region<'c>,
    ) -> Self {
        Self { state, env, region }
    }

    /// Emits MLIR for a statement.
    ///
    /// Returns `Some(block)` as the continuation block for the next statement,
    /// or `None` if control flow was terminated.
    pub fn emit(
        &mut self,
        stmt: &Statement,
        block: BlockRef<'c, 'b>,
    ) -> anyhow::Result<Option<BlockRef<'c, 'b>>> {
        match stmt {
            Statement::VariableDeclarationStatement(decl) => {
                self.emit_variable_declaration(decl, block)
            }
            Statement::ExpressionStatement(expr_stmt) => {
                let emitter = ExpressionEmitter::new(self.state, self.env, self.region);
                let (_val, block) = emitter.emit(&expr_stmt.expression, block)?;
                Ok(Some(block))
            }
            Statement::ReturnStatement(ret) => self.emit_return(ret, block),
            Statement::IfStatement(if_stmt) => self.emit_if(if_stmt, block),
            Statement::ForStatement(for_stmt) => self.emit_for(for_stmt, block),
            Statement::WhileStatement(while_stmt) => self.emit_while(while_stmt, block),
            Statement::DoWhileStatement(do_while) => self.emit_do_while(do_while, block),
            Statement::BreakStatement(_) => self.emit_break(block),
            Statement::ContinueStatement(_) => self.emit_continue(block),
            Statement::Block(inner) => self.emit_block(&inner.statements, block),
            Statement::UncheckedBlock(inner) => self.emit_block(&inner.block.statements, block),
            _ => anyhow::bail!("unsupported statement: {stmt:?}"),
        }
    }

    /// Emits a sequence of statements.
    fn emit_block(
        &mut self,
        stmts: &[Statement],
        block: BlockRef<'c, 'b>,
    ) -> anyhow::Result<Option<BlockRef<'c, 'b>>> {
        let mut current = block;
        for stmt in stmts {
            match self.emit(stmt, current)? {
                Some(next) => current = next,
                None => return Ok(None),
            }
        }
        Ok(Some(current))
    }

    /// Emits a variable declaration with optional initializer.
    fn emit_variable_declaration(
        &mut self,
        decl: &slang_solidity::backend::ir::ir2_flat_contracts::VariableDeclarationStatement,
        block: BlockRef<'c, 'b>,
    ) -> anyhow::Result<Option<BlockRef<'c, 'b>>> {
        let name = decl.name.text.as_str();

        let emitter = ExpressionEmitter::new(self.state, self.env, self.region);
        let ptr = emitter.emit_alloca(&block);

        if let Some(ref init_expr) = decl.value {
            let (init_val, block) = emitter.emit(init_expr, block)?;
            emitter.emit_store(init_val, ptr, &block)?;
            if decl.type_name.as_ref().is_some_and(|t| TypeMapper::is_signed(t)) {
                self.env.mark_signed(name);
            }
            self.env.define_variable(name.to_owned(), ptr);
            Ok(Some(block))
        } else {
            let zero = emitter.emit_i256_constant(0, &block);
            emitter.emit_store(zero, ptr, &block)?;
            if decl.type_name.as_ref().is_some_and(|t| TypeMapper::is_signed(t)) {
                self.env.mark_signed(name);
            }
            self.env.define_variable(name.to_owned(), ptr);
            Ok(Some(block))
        }
    }

    /// Emits a return statement.
    fn emit_return(
        &mut self,
        ret: &slang_solidity::backend::ir::ir2_flat_contracts::ReturnStatement,
        block: BlockRef<'c, 'b>,
    ) -> anyhow::Result<Option<BlockRef<'c, 'b>>> {
        let location = self.state.location();

        if let Some(ref expr) = ret.expression {
            let emitter = ExpressionEmitter::new(self.state, self.env, self.region);
            let (value, block) = emitter.emit(expr, block)?;
            block.append_operation(
                melior::ir::operation::OperationBuilder::new(ops::RETURN, location)
                    .add_operands(&[value])
                    .build()
                    .expect("valid llvm.return"),
            );
        } else {
            block.append_operation(
                melior::ir::operation::OperationBuilder::new(ops::RETURN, location)
                    .build()
                    .expect("valid llvm.return"),
            );
        }

        Ok(None)
    }

    /// Emits an if/else statement with conditional branching.
    fn emit_if(
        &mut self,
        if_stmt: &slang_solidity::backend::ir::ir2_flat_contracts::IfStatement,
        block: BlockRef<'c, 'b>,
    ) -> anyhow::Result<Option<BlockRef<'c, 'b>>> {
        let emitter = ExpressionEmitter::new(self.state, self.env, self.region);
        let (cond_val, block) = emitter.emit(&if_stmt.condition, block)?;
        let cond_bool = emitter.emit_is_nonzero(cond_val, &block);

        let then_block = self.region.append_block(Block::new(&[]));
        let merge_block = self.region.append_block(Block::new(&[]));

        if let Some(ref else_stmt) = if_stmt.else_branch {
            let else_block = self.region.append_block(Block::new(&[]));
            block.append_operation(
                self.state.llvm_cond_br(cond_bool, &then_block, &else_block, &[], &[]),
            );

            let then_end = self.emit(&if_stmt.body, then_block)?;
            if let Some(then_end) = then_end {
                then_end.append_operation(self.state.llvm_br(&merge_block, &[]));
            }

            let else_end = self.emit(else_stmt, else_block)?;
            if let Some(else_end) = else_end {
                else_end.append_operation(self.state.llvm_br(&merge_block, &[]));
            }

            if then_end.is_some() || else_end.is_some() {
                Ok(Some(merge_block))
            } else {
                Ok(None)
            }
        } else {
            block.append_operation(
                self.state.llvm_cond_br(cond_bool, &then_block, &merge_block, &[], &[]),
            );

            let then_end = self.emit(&if_stmt.body, then_block)?;
            if let Some(then_end) = then_end {
                then_end.append_operation(self.state.llvm_br(&merge_block, &[]));
            }

            Ok(Some(merge_block))
        }
    }

    /// Emits a for loop.
    fn emit_for(
        &mut self,
        for_stmt: &slang_solidity::backend::ir::ir2_flat_contracts::ForStatement,
        block: BlockRef<'c, 'b>,
    ) -> anyhow::Result<Option<BlockRef<'c, 'b>>> {
        // Emit initialization.
        let block = match &for_stmt.initialization {
            ForStatementInitialization::VariableDeclarationStatement(decl) => {
                match self.emit(&Statement::VariableDeclarationStatement(decl.clone()), block)? {
                    Some(b) => b,
                    None => return Ok(None),
                }
            }
            ForStatementInitialization::ExpressionStatement(expr_stmt) => {
                match self.emit(&Statement::ExpressionStatement(expr_stmt.clone()), block)? {
                    Some(b) => b,
                    None => return Ok(None),
                }
            }
            ForStatementInitialization::TupleDeconstructionStatement(_) => {
                anyhow::bail!("tuple deconstruction in for-init not yet supported")
            }
            ForStatementInitialization::Semicolon => block,
        };

        let cond_block = self.region.append_block(Block::new(&[]));
        let body_block = self.region.append_block(Block::new(&[]));
        let iter_block = self.region.append_block(Block::new(&[]));
        let exit_block = self.region.append_block(Block::new(&[]));

        block.append_operation(self.state.llvm_br(&cond_block, &[]));

        // Condition.
        match &for_stmt.condition {
            ForStatementCondition::ExpressionStatement(expr_stmt) => {
                let emitter = ExpressionEmitter::new(self.state, self.env, self.region);
                let (cond_val, cond_block_end) =
                    emitter.emit(&expr_stmt.expression, cond_block)?;
                let cond_bool = emitter.emit_is_nonzero(cond_val, &cond_block_end);
                cond_block_end.append_operation(
                    self.state.llvm_cond_br(cond_bool, &body_block, &exit_block, &[], &[]),
                );
            }
            ForStatementCondition::Semicolon => {
                cond_block.append_operation(self.state.llvm_br(&body_block, &[]));
            }
        }

        // Body with loop targets.
        self.env.push_loop(LoopTarget {
            break_block: exit_block,
            continue_block: iter_block,
        });
        let body_end = self.emit(&for_stmt.body, body_block)?;
        self.env.pop_loop();

        if let Some(body_end) = body_end {
            body_end.append_operation(self.state.llvm_br(&iter_block, &[]));
        }

        // Iterator.
        if let Some(ref iter_expr) = for_stmt.iterator {
            let expr_emitter = ExpressionEmitter::new(self.state, self.env, self.region);
            let (_val, iter_end) = expr_emitter.emit(iter_expr, iter_block)?;
            iter_end.append_operation(self.state.llvm_br(&cond_block, &[]));
        } else {
            iter_block.append_operation(self.state.llvm_br(&cond_block, &[]));
        }

        Ok(Some(exit_block))
    }

    /// Emits a while loop.
    fn emit_while(
        &mut self,
        while_stmt: &slang_solidity::backend::ir::ir2_flat_contracts::WhileStatement,
        block: BlockRef<'c, 'b>,
    ) -> anyhow::Result<Option<BlockRef<'c, 'b>>> {
        let cond_block = self.region.append_block(Block::new(&[]));
        let body_block = self.region.append_block(Block::new(&[]));
        let exit_block = self.region.append_block(Block::new(&[]));

        block.append_operation(self.state.llvm_br(&cond_block, &[]));

        let emitter = ExpressionEmitter::new(self.state, self.env, self.region);
        let (cond_val, cond_end) = emitter.emit(&while_stmt.condition, cond_block)?;
        let cond_bool = emitter.emit_is_nonzero(cond_val, &cond_end);
        cond_end.append_operation(
            self.state.llvm_cond_br(cond_bool, &body_block, &exit_block, &[], &[]),
        );

        self.env.push_loop(LoopTarget {
            break_block: exit_block,
            continue_block: cond_block,
        });
        let body_end = self.emit(&while_stmt.body, body_block)?;
        self.env.pop_loop();

        if let Some(body_end) = body_end {
            body_end.append_operation(self.state.llvm_br(&cond_block, &[]));
        }

        Ok(Some(exit_block))
    }

    /// Emits a do-while loop.
    fn emit_do_while(
        &mut self,
        do_while: &slang_solidity::backend::ir::ir2_flat_contracts::DoWhileStatement,
        block: BlockRef<'c, 'b>,
    ) -> anyhow::Result<Option<BlockRef<'c, 'b>>> {
        let body_block = self.region.append_block(Block::new(&[]));
        let cond_block = self.region.append_block(Block::new(&[]));
        let exit_block = self.region.append_block(Block::new(&[]));

        block.append_operation(self.state.llvm_br(&body_block, &[]));

        self.env.push_loop(LoopTarget {
            break_block: exit_block,
            continue_block: cond_block,
        });
        let body_end = self.emit(&do_while.body, body_block)?;
        self.env.pop_loop();

        if let Some(body_end) = body_end {
            body_end.append_operation(self.state.llvm_br(&cond_block, &[]));
        }

        let emitter = ExpressionEmitter::new(self.state, self.env, self.region);
        let (cond_val, cond_end) = emitter.emit(&do_while.condition, cond_block)?;
        let cond_bool = emitter.emit_is_nonzero(cond_val, &cond_end);
        cond_end.append_operation(
            self.state.llvm_cond_br(cond_bool, &body_block, &exit_block, &[], &[]),
        );

        Ok(Some(exit_block))
    }

    /// Emits a break statement.
    fn emit_break(&self, block: BlockRef<'c, 'b>) -> anyhow::Result<Option<BlockRef<'c, 'b>>> {
        let target = self.env.current_loop().ok_or_else(|| {
            anyhow::anyhow!("break outside of loop")
        })?;
        block.append_operation(self.state.llvm_br(&target.break_block, &[]));
        Ok(None)
    }

    /// Emits a continue statement.
    fn emit_continue(
        &self,
        block: BlockRef<'c, 'b>,
    ) -> anyhow::Result<Option<BlockRef<'c, 'b>>> {
        let target = self.env.current_loop().ok_or_else(|| {
            anyhow::anyhow!("continue outside of loop")
        })?;
        block.append_operation(self.state.llvm_br(&target.continue_block, &[]));
        Ok(None)
    }
}
