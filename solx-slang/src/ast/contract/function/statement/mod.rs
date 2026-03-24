//!
//! Statement lowering to MLIR operations.
//!

pub mod control_flow;

use std::collections::HashMap;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Region;
use slang_solidity::backend::ir::ast::Statement;
use slang_solidity::backend::ir::ast::Statements;
use slang_solidity::cst::NodeId;

use solx_mlir::Context;
use solx_mlir::Environment;

use crate::ast::contract::function::expression::ExpressionEmitter;

/// Lowers Solidity statements to MLIR operations with control flow.
///
/// Returns `Some(block)` as the continuation block, or `None` when control
/// flow has been terminated (by `return`, `break`, or `continue`).
pub struct StatementEmitter<'state, 'context, 'block> {
    /// The shared MLIR context.
    pub state: &'state Context<'context>,
    /// Variable environment (mutable for new declarations and loop targets).
    pub environment: &'state mut Environment<'context, 'block>,
    /// The current region for creating new blocks.
    /// Stored as a raw pointer to allow switching between Sol op regions
    /// without lifetime conflicts.
    region_pointer: *const Region<'context>,
    /// State variable node ID to storage slot mapping.
    pub storage_layout: &'state HashMap<NodeId, u64>,
}

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Creates a new statement emitter.
    pub fn new(
        state: &'state Context<'context>,
        environment: &'state mut Environment<'context, 'block>,
        region: &Region<'context>,
        storage_layout: &'state HashMap<NodeId, u64>,
    ) -> Self {
        Self {
            state,
            environment,
            region_pointer: region as *const Region<'context>,
            storage_layout,
        }
    }

    /// Returns a reference to the current region.
    ///
    /// # Safety
    ///
    /// The region pointer is valid as long as the MLIR module exists,
    /// which outlives all emitters.
    pub fn region(&self) -> &Region<'context> {
        // SAFETY: The region is owned by the MLIR module and outlives this emitter.
        unsafe { &*self.region_pointer }
    }

    /// Switches the current region for emitting into Sol op regions.
    pub fn set_region(&mut self, region: &Region<'context>) {
        self.region_pointer = region as *const Region<'context>;
    }

    /// Emits MLIR for a statement.
    ///
    /// Returns `Some(block)` as the continuation block for the next statement,
    /// or `None` if control flow was terminated.
    ///
    /// # Errors
    ///
    /// Returns an error if the statement contains unsupported constructs.
    pub fn emit(
        &mut self,
        statement: &Statement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        match statement {
            Statement::VariableDeclarationStatement(declaration) => {
                self.emit_variable_declaration(declaration, block)
            }
            Statement::ExpressionStatement(expression_statement) => {
                let expression = expression_statement.expression();
                let emitter =
                    ExpressionEmitter::new(self.state, self.environment, self.storage_layout);
                let (_value, block) = emitter.emit(&expression, block)?;
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
            // TODO: thread checked/unchecked flag to use different arithmetic ops
            Statement::UncheckedBlock(inner) => self.emit_block(inner.block().statements(), block),
            Statement::RevertStatement(_revert) => {
                // TODO: encode custom error data from revert arguments
                self.state.builder.emit_sol_revert(&block);
                // TODO(sol-dialect): remove once sol.revert is marked IsTerminator
                block.append_operation(melior::dialect::llvm::unreachable(
                    self.state.builder.unknown_location,
                ));
                Ok(None)
            }
            _ => anyhow::bail!(
                "unsupported statement: {:?}",
                std::mem::discriminant(statement)
            ),
        }
    }

    /// Emits a sequence of statements inside a new lexical scope.
    ///
    /// # Errors
    ///
    /// Returns an error if any statement contains unsupported constructs.
    pub fn emit_block(
        &mut self,
        statements: Statements,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        self.environment.enter_scope();
        let mut current = block;
        for statement in statements.iter() {
            match self.emit(&statement, current)? {
                Some(next) => current = next,
                None => {
                    self.environment.exit_scope();
                    return Ok(None);
                }
            }
        }
        self.environment.exit_scope();
        Ok(Some(current))
    }

    /// Emits a variable declaration with optional initializer.
    fn emit_variable_declaration(
        &mut self,
        declaration: &slang_solidity::backend::ir::ast::VariableDeclarationStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let name = declaration.name().name();

        let emitter = ExpressionEmitter::new(self.state, self.environment, self.storage_layout);
        let pointer = emitter.state.builder.emit_sol_alloca(&block);

        let block = if let Some(ref initializer_expression) = declaration.value() {
            let (initial_value, block) = emitter.emit(initializer_expression, block)?;
            emitter
                .state
                .builder
                .emit_sol_store(initial_value, pointer, &block);
            block
        } else {
            let zero = self.state.builder.emit_sol_constant(0, &block);
            emitter.state.builder.emit_sol_store(zero, pointer, &block);
            block
        };

        self.environment.define_variable(name, pointer);
        Ok(Some(block))
    }

    /// Emits a `sol.break` terminator.
    fn emit_break(
        &self,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        self.state.builder.emit_sol_break(&block);
        Ok(None)
    }

    /// Emits a `sol.continue` terminator.
    fn emit_continue(
        &self,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        self.state.builder.emit_sol_continue(&block);
        Ok(None)
    }

    /// Emits a return statement.
    fn emit_return(
        &mut self,
        return_statement: &slang_solidity::backend::ir::ast::ReturnStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        if let Some(ref expression) = return_statement.expression() {
            let emitter = ExpressionEmitter::new(self.state, self.environment, self.storage_layout);
            let (value, block) = emitter.emit(expression, block)?;
            self.state.builder.emit_sol_return(&[value], &block);
        } else {
            self.state.builder.emit_sol_return(&[], &block);
        }

        Ok(None)
    }
}
