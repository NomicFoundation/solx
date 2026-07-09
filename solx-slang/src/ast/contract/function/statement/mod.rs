//!
//! Statement lowering to MLIR operations.
//!

pub mod control_flow;
pub mod event;
pub mod revert;
pub mod variable_declaration;

use std::collections::HashMap;

use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Statement;
use slang_solidity_v2::ast::Statements;

use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::Type;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::storage_slot::StorageSlot;

/// Lowers Solidity statements to MLIR operations with control flow.
///
/// Statements append at the insertion cursor and reposition it; `return`,
/// `break`, and `continue` terminate the block the cursor points at.
pub struct StatementEmitter<'state, 'context> {
    /// Variable environment (mutable for new declarations and loop targets).
    environment: &'state mut Environment<'context>,
    /// State variable node ID to storage slot mapping.
    storage_layout: &'state HashMap<NodeId, StorageSlot>,
    /// The function's declared return types, for `emit_return` to cast to.
    return_types: &'state [Type<'context>],
    /// Whether arithmetic operations use checked variants (`sol.cadd` etc.).
    ///
    /// `true` by default. Set to `false` inside `unchecked {}` blocks.
    checked: bool,
}

impl<'state, 'context> StatementEmitter<'state, 'context> {
    /// Creates a new statement emitter.
    pub fn new(
        environment: &'state mut Environment<'context>,
        storage_layout: &'state HashMap<NodeId, StorageSlot>,
        return_types: &'state [Type<'context>],
    ) -> Self {
        Self {
            environment,
            storage_layout,
            return_types,
            checked: true,
        }
    }

    /// Emits MLIR for a statement, appending at and repositioning the insertion cursor.
    ///
    /// # Errors
    ///
    /// Returns an error if the statement contains unsupported constructs.
    pub fn emit(
        &mut self,
        statement: &Statement,
        context: &mut Context<'context>,
    ) -> anyhow::Result<()> {
        match statement {
            Statement::VariableDeclarationStatement(declaration) => {
                self.emit_variable_declaration(declaration, context)
            }
            Statement::ExpressionStatement(expression_statement) => {
                let expression = expression_statement.expression();
                if let Expression::FunctionCallExpression(call) = &expression
                    && let Expression::Identifier(identifier) = call.operand()
                    && identifier.name() == revert::IDENTIFIER
                {
                    return self.emit_revert_call(call, context);
                }
                let emitter =
                    ExpressionEmitter::new(self.environment, self.storage_layout, self.checked);
                emitter.emit(&expression, context)?;
                Ok(())
            }
            Statement::ReturnStatement(return_statement) => {
                self.emit_return(return_statement, context)
            }
            Statement::IfStatement(if_statement) => self.emit_if(if_statement, context),
            Statement::ForStatement(for_statement) => self.emit_for(for_statement, context),
            Statement::WhileStatement(while_statement) => self.emit_while(while_statement, context),
            Statement::DoWhileStatement(do_while) => self.emit_do_while(do_while, context),
            Statement::BreakStatement(_) => self.emit_break(context),
            Statement::ContinueStatement(_) => self.emit_continue(context),
            Statement::Block(inner) => self.emit_block(inner.statements(), context),
            Statement::UncheckedBlock(inner) => {
                let saved_checked = self.checked;
                self.checked = false;
                let result = self.emit_block(inner.block().statements(), context);
                self.checked = saved_checked;
                result
            }
            Statement::RevertStatement(revert) => self.emit_revert(revert, context),
            Statement::EmitStatement(emit_statement) => self.emit_event(emit_statement, context),
            _ => anyhow::bail!(
                "unsupported statement: {:?}",
                std::mem::discriminant(statement)
            ),
        }
    }

    /// Emits a sequence of statements inside a new lexical scope, stopping once a statement
    /// terminates the current block.
    ///
    /// # Errors
    ///
    /// Returns an error if any statement contains unsupported constructs.
    pub fn emit_block(
        &mut self,
        statements: Statements,
        context: &mut Context<'context>,
    ) -> anyhow::Result<()> {
        self.environment.enter_scope();
        for statement in statements.iter() {
            self.emit(&statement, context)?;
            if context.current_block().is_terminated() {
                break;
            }
        }
        self.environment.exit_scope();
        Ok(())
    }

    /// Emits a `sol.break` terminator.
    fn emit_break(&self, context: &mut Context<'context>) -> anyhow::Result<()> {
        let block = context.current_block();
        block.r#break(context);
        Ok(())
    }

    /// Emits a `sol.continue` terminator.
    fn emit_continue(&self, context: &mut Context<'context>) -> anyhow::Result<()> {
        let block = context.current_block();
        block.r#continue(context);
        Ok(())
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
        context: &mut Context<'context>,
    ) -> anyhow::Result<()> {
        let Some(expression) = return_statement.expression() else {
            let block = context.current_block();
            block.r#return(&[], context);
            return Ok(());
        };

        let emitter = ExpressionEmitter::new(self.environment, self.storage_layout, self.checked);

        let values = if let Expression::TupleExpression(tuple) = &expression
            && tuple.items().len() > 1
        {
            let items = tuple.items();
            let mut values = Vec::with_capacity(items.len());
            for item in items.iter() {
                let inner = item
                    .expression()
                    .ok_or_else(|| anyhow::anyhow!("empty tuple element in return"))?;
                let value = emitter.emit_value(&inner, context)?;
                values.push(value);
            }
            values
        } else {
            vec![emitter.emit_value(&expression, context)?]
        };

        let cast_values: Vec<_> = values
            .into_iter()
            .zip(self.return_types.iter())
            .map(|(value, &return_type)| {
                TypeConversion::from_target_type(return_type, context).emit(value, context)
            })
            .collect();

        let block = context.current_block();
        block.r#return(&cast_values, context);
        Ok(())
    }
}
