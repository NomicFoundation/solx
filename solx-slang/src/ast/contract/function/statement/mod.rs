//!
//! Statement lowering to MLIR operations.
//!

pub mod assembly;
pub mod control_flow;
pub mod event;
pub mod revert;
pub mod try_statement;
pub mod variable_declaration;

use std::collections::HashMap;

use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::Type;
use ruint::aliases::U256;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Statement;
use slang_solidity_v2::ast::Statements;

use solx_mlir::Context;
use solx_mlir::Environment;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::storage_slot::StorageSlot;

/// Lowers Solidity statements to MLIR operations with control flow.
///
/// Returns `Some(block)` as the continuation block, or `None` when control
/// flow has been terminated (by `return`, `break`, or `continue`).
pub struct StatementEmitter<'state, 'context, 'block> {
    /// The shared MLIR context.
    state: &'state Context<'context>,
    /// Variable environment (mutable for new declarations and loop targets).
    environment: &'state mut Environment<'context, 'block>,
    /// The current region for creating new blocks.
    /// Stored as a raw pointer to allow switching between Sol op regions
    /// without lifetime conflicts.
    region_pointer: *const Region<'context>,
    /// State variable node ID to storage slot mapping.
    storage_layout: &'state HashMap<NodeId, U256>,
    /// The function's declared return types, for `emit_return` to cast to.
    return_types: &'state [Type<'context>],
    /// Whether arithmetic operations use checked variants (`sol.cadd` etc.).
    ///
    /// `true` by default. Set to `false` inside `unchecked {}` blocks.
    checked: bool,
    /// Yul function definitions visible to the current assembly emission.
    /// At call sites, we inline the body — `yul.func` cannot live inside
    /// `sol.func` (not a `SymbolTable` region), and hoisting it to
    /// `sol.contract` requires architectural changes we have not made.
    pub(super) yul_functions:
        HashMap<String, slang_solidity_v2::ast::YulFunctionDefinition>,
    /// Depth counter used to abort runaway inlining of recursive yul fns.
    pub(super) yul_inline_depth: HashMap<String, usize>,
    /// Inlined modifier/body stages. Stage `i` is a modifier body; the last
    /// stage is the wrapped function body. Each `_;` placeholder emits the
    /// next stage. Empty when the function has no modifiers (the body is
    /// emitted directly by `FunctionEmitter`).
    pub(super) modifier_stages: Vec<Statements>,
    /// Index of the next stage to emit when a `_;` placeholder is hit.
    pub(super) modifier_stage_index: usize,
}

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Creates a new statement emitter.
    pub fn new(
        state: &'state Context<'context>,
        environment: &'state mut Environment<'context, 'block>,
        region: &Region<'context>,
        storage_layout: &'state HashMap<NodeId, U256>,
        return_types: &'state [Type<'context>],
    ) -> Self {
        Self {
            state,
            environment,
            region_pointer: region as *const Region<'context>,
            storage_layout,
            return_types,
            checked: true,
            yul_functions: HashMap::new(),
            yul_inline_depth: HashMap::new(),
            modifier_stages: Vec::new(),
            modifier_stage_index: 0,
        }
    }

    /// Emits the modifier chain starting at the current stage index. Stage 0
    /// is the outermost modifier body; subsequent stages are reached via the
    /// `_;` placeholder. Used by `FunctionEmitter` to drive a modified
    /// function: it binds modifier parameters, sets `modifier_stages`, and
    /// calls this with the entry block.
    pub fn emit_modifier_chain(
        &mut self,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let stage = self.modifier_stage_index;
        let Some(statements) = self.modifier_stages.get(stage).cloned() else {
            return Ok(Some(block));
        };
        self.modifier_stage_index = stage + 1;
        let result = self.emit_block(statements, block);
        self.modifier_stage_index = stage;
        result
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
                // A bare `_;` inside a modifier body is the placeholder for
                // the wrapped function body (or the next modifier).
                if let Expression::Identifier(identifier) = &expression
                    && matches!(
                        identifier.resolve_to_built_in(),
                        Some(slang_solidity_v2::ast::BuiltIn::ModifierUnderscore)
                    )
                {
                    return self.emit_modifier_chain(block);
                }
                if let Expression::FunctionCallExpression(call) = &expression
                    && let Expression::Identifier(identifier) = call.operand()
                    && identifier.name() == revert::IDENTIFIER
                {
                    return self.emit_revert_call(call, block);
                }
                let emitter = ExpressionEmitter::new(
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
            Statement::AssemblyStatement(assembly) => self.emit_assembly(assembly, block),
            Statement::TryStatement(try_statement) => self.emit_try(try_statement, block),
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
        return_statement: &slang_solidity_v2::ast::ReturnStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let Some(expression) = return_statement.expression() else {
            self.state.builder.emit_sol_return(&[], &block);
            return Ok(None);
        };

        let emitter = ExpressionEmitter::new(
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

        let cast_values: Vec<_> = values
            .into_iter()
            .zip(self.return_types.iter())
            .map(|(value, &return_type)| {
                TypeConversion::from_target_type(return_type, &self.state.builder).emit(
                    value,
                    &self.state.builder,
                    &block,
                )
            })
            .collect();

        self.state.builder.emit_sol_return(&cast_values, &block);
        Ok(None)
    }
}
