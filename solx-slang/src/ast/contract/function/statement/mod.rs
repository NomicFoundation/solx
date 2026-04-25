//!
//! Statement lowering to MLIR operations.
//!

pub mod control_flow;

use std::collections::HashMap;
use std::rc::Rc;

use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::Type;
use slang_solidity::backend::SemanticAnalysis;
use slang_solidity::backend::ir::ast::Statement;
use slang_solidity::backend::ir::ast::Statements;
use slang_solidity::cst::NodeId;

use solx_mlir::Context;
use solx_mlir::Environment;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

/// Lowers Solidity statements to MLIR operations with control flow.
///
/// Returns `Some(block)` as the continuation block, or `None` when control
/// flow has been terminated (by `return`, `break`, or `continue`).
pub struct StatementEmitter<'state, 'context, 'block> {
    /// Slang semantic analysis for resolving expression types.
    semantic: Rc<SemanticAnalysis>,
    /// The shared MLIR context.
    state: &'state Context<'context>,
    /// Variable environment (mutable for new declarations and loop targets).
    environment: &'state mut Environment<'context, 'block>,
    /// The current region for creating new blocks.
    /// Stored as a raw pointer to allow switching between Sol op regions
    /// without lifetime conflicts.
    region_pointer: *const Region<'context>,
    /// State variable node ID to storage slot mapping.
    storage_layout: &'state HashMap<NodeId, u64>,
    /// The function's declared return types, for `emit_return` to cast to.
    return_types: &'state [Type<'context>],
    /// Whether arithmetic operations use checked variants (`sol.cadd` etc.).
    ///
    /// `true` by default. Set to `false` inside `unchecked {}` blocks.
    checked: bool,
}

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Creates a new statement emitter.
    pub fn new(
        semantic: &Rc<SemanticAnalysis>,
        state: &'state Context<'context>,
        environment: &'state mut Environment<'context, 'block>,
        region: &Region<'context>,
        storage_layout: &'state HashMap<NodeId, u64>,
        return_types: &'state [Type<'context>],
    ) -> Self {
        Self {
            semantic: Rc::clone(semantic),
            state,
            environment,
            region_pointer: region as *const Region<'context>,
            storage_layout,
            return_types,
            checked: true,
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
                let emitter = ExpressionEmitter::new(
                    &self.semantic,
                    self.state,
                    self.environment,
                    self.storage_layout,
                    self.checked,
                );
                let (_, block) = emitter.emit(&expression, block)?;
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
            Statement::UncheckedBlock(inner) => {
                let saved_checked = self.checked;
                self.checked = false;
                let result = self.emit_block(inner.block().statements(), block);
                self.checked = saved_checked;
                result
            }
            Statement::RevertStatement(_revert) => {
                // TODO: encode custom error data from revert arguments
                self.state.builder.emit_sol_revert(&block);
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
        let declared_type = declaration
            .get_type()
            .map(|slang_type| TypeConversion::resolve_slang_type(&slang_type, &self.state.builder))
            .unwrap_or_else(|| self.state.builder.types.ui256);

        let emitter = ExpressionEmitter::new(
            &self.semantic,
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );

        // For explicit initializers, evaluate and cast before alloca to match
        // solc's emission order (constant → cast → alloca → store).
        // For implicit zero-initialization, alloca is emitted first.
        let (block, initial_value) = if let Some(ref initializer_expression) = declaration.value() {
            let (initial_value, block) = emitter.emit_value(initializer_expression, block)?;
            let cast_value =
                emitter
                    .state
                    .builder
                    .emit_sol_cast(initial_value, declared_type, &block);
            (block, Some(cast_value))
        } else {
            (block, None)
        };

        let pointer = emitter.state.builder.emit_sol_alloca(declared_type, &block);

        let stored_value = initial_value.unwrap_or_else(|| {
            self.state
                .builder
                .emit_sol_constant(0, declared_type, &block)
        });
        emitter
            .state
            .builder
            .emit_sol_store(stored_value, pointer, &block);

        self.environment
            .define_variable(name, pointer, declared_type);
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
            // TODO: support multi-return functions (tuple deconstruction).
            anyhow::ensure!(
                self.return_types.len() <= 1,
                "multi-return functions are not yet supported"
            );
            let emitter = ExpressionEmitter::new(
                &self.semantic,
                self.state,
                self.environment,
                self.storage_layout,
                self.checked,
            );
            let (value, block) = emitter.emit_value(expression, block)?;
            let return_type = self
                .return_types
                .first()
                .copied()
                .unwrap_or_else(|| self.state.builder.types.ui256);
            let return_value = self.state.builder.emit_sol_cast(value, return_type, &block);
            self.state.builder.emit_sol_return(&[return_value], &block);
        } else {
            self.state.builder.emit_sol_return(&[], &block);
        }

        Ok(None)
    }
}
