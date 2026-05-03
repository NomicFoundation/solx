//!
//! Statement lowering to MLIR operations.
//!

pub mod control_flow;
pub mod event;
pub mod revert;

use std::collections::HashMap;
use std::rc::Rc;

use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::Type;
use slang_solidity::backend::SemanticAnalysis;
use slang_solidity::backend::ir::ast::Expression;
use slang_solidity::backend::ir::ast::Statement;
use slang_solidity::backend::ir::ast::Statements;
use slang_solidity::backend::ir::ast::TupleDeconstructionMember;
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
                if let Expression::FunctionCallExpression(call) = &expression
                    && let Expression::Identifier(identifier) = call.operand()
                    && identifier.name() == revert::IDENTIFIER
                {
                    return self.emit_revert_call(call, block);
                }
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
            Statement::RevertStatement(revert) => self.emit_revert(revert, block),
            Statement::EmitStatement(emit_statement) => self.emit_event(emit_statement, block),
            Statement::TupleDeconstructionStatement(deconstruction) => {
                self.emit_tuple_deconstruction(deconstruction, block)
            }
            _ => anyhow::bail!(
                "unsupported statement: {:?}",
                std::mem::discriminant(statement)
            ),
        }
    }

    /// Emits a tuple deconstruction statement of the form
    /// `(decl_or_id_or_skip, ...) = (rhs0, rhs1, ...)`.
    ///
    /// The right-hand side must currently be a tuple expression; each item is
    /// emitted independently, then assigned to the corresponding LHS slot.
    /// `None` slots discard their value, `Identifier` slots store into an
    /// existing variable, and `VariableDeclarationStatement` slots allocate a
    /// new variable. Multi-result function calls on the RHS are not yet
    /// supported.
    fn emit_tuple_deconstruction(
        &mut self,
        statement: &slang_solidity::backend::ir::ast::TupleDeconstructionStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let expression = statement.expression();
        let Expression::TupleExpression(tuple) = &expression else {
            anyhow::bail!(
                "tuple deconstruction with non-tuple right-hand side is not yet supported"
            );
        };

        let items = tuple.items();
        let members = statement.members();
        anyhow::ensure!(
            items.len() == members.len(),
            "tuple deconstruction arity mismatch: {} LHS slots vs {} RHS values",
            members.len(),
            items.len(),
        );

        let emitter = ExpressionEmitter::new(
            &self.semantic,
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );

        let mut values = Vec::with_capacity(items.len());
        let mut current = block;
        for item in items.iter() {
            let inner = item
                .expression()
                .ok_or_else(|| anyhow::anyhow!("empty tuple element on RHS of deconstruction"))?;
            let (value, next) = emitter.emit_value(&inner, current)?;
            values.push(value);
            current = next;
        }

        for (member, value) in members.iter().zip(values.into_iter()) {
            match member {
                TupleDeconstructionMember::None => {
                    // Discard the value; nothing to bind.
                }
                TupleDeconstructionMember::Identifier(identifier) => {
                    let name = identifier.name();
                    let (pointer, target_type) = self
                        .environment
                        .variable_with_type(&name)
                        .ok_or_else(|| anyhow::anyhow!("undefined variable: {name}"))?;
                    let cast = self
                        .state
                        .builder
                        .emit_sol_cast(value, target_type, &current);
                    self.state.builder.emit_sol_store(cast, pointer, &current);
                }
                TupleDeconstructionMember::VariableDeclarationStatement(declaration) => {
                    let name = declaration.name().name();
                    let declared_type = declaration
                        .get_type()
                        .map(|slang_type| {
                            TypeConversion::resolve_slang_type(&slang_type, &self.state.builder)
                        })
                        .unwrap_or_else(|| self.state.builder.types.ui256);
                    let cast = self
                        .state
                        .builder
                        .emit_sol_cast(value, declared_type, &current);
                    let pointer = self.state.builder.emit_sol_alloca(declared_type, &current);
                    self.state.builder.emit_sol_store(cast, pointer, &current);
                    self.environment
                        .define_variable(name, pointer, declared_type);
                }
            }
        }

        Ok(Some(current))
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
    ///
    /// A multi-element tuple expression in the return position is unpacked
    /// into one value per declared return slot; any other expression yields
    /// a single value. Each value is cast to its corresponding declared
    /// return type before being emitted as a `sol.return` operand.
    fn emit_return(
        &mut self,
        return_statement: &slang_solidity::backend::ir::ast::ReturnStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let Some(expression) = return_statement.expression() else {
            self.state.builder.emit_sol_return(&[], &block);
            return Ok(None);
        };

        let emitter = ExpressionEmitter::new(
            &self.semantic,
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );

        let (values, block) = if let Expression::TupleExpression(tuple) = &expression
            && tuple.items().len() > 1
        {
            let items = tuple.items();
            let mut values = Vec::with_capacity(items.len());
            let mut current = block;
            for item in items.iter() {
                let inner = item
                    .expression()
                    .ok_or_else(|| anyhow::anyhow!("empty tuple element in return"))?;
                let (value, next) = emitter.emit_value(&inner, current)?;
                values.push(value);
                current = next;
            }
            (values, current)
        } else {
            let (value, block) = emitter.emit_value(&expression, block)?;
            (vec![value], block)
        };

        anyhow::ensure!(
            values.len() == self.return_types.len(),
            "return value count {} does not match function return arity {}",
            values.len(),
            self.return_types.len(),
        );

        let cast_values: Vec<_> = values
            .into_iter()
            .zip(self.return_types.iter())
            .map(|(value, &return_type)| {
                self.state.builder.emit_sol_cast(value, return_type, &block)
            })
            .collect();

        self.state.builder.emit_sol_return(&cast_values, &block);
        Ok(None)
    }
}
