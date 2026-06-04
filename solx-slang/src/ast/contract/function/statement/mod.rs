//!
//! Statement lowering to MLIR operations.
//!

/// Control flow statement lowering (`if`, `for`, `while`, `do`/`while`).
pub mod control_flow;
/// Event emit statement lowering.
pub mod event;
/// Expression statement lowering.
pub mod expression_statement;
/// Named call-argument ordering.
pub mod named_arguments;
/// Return statement lowering.
pub mod return_statement;
/// Revert statement lowering.
pub mod revert;
/// Local variable declaration statement lowering.
pub mod variable_declaration;

use std::collections::HashMap;

use melior::ir::BlockRef;
use melior::ir::Type;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Statement;

use solx_mlir::Context;
use solx_mlir::Environment;

use crate::ast::contract::function::storage_slot::StorageSlot;

/// Lowers Solidity statements to MLIR operations with control flow.
///
/// Returns `Some(block)` as the continuation block, or `None` when control
/// flow has been terminated (by `return`, `break`, or `continue`).
// TODO(skeleton): the fields are written but not yet read; they become read as
// the statement-handler domains are filled in.
#[allow(dead_code)]
pub struct StatementEmitter<'state, 'context, 'block> {
    /// The shared MLIR context.
    state: &'state Context<'context>,
    /// Variable environment (mutable for new declarations and loop targets).
    environment: &'state mut Environment<'context, 'block>,
    /// State variable node ID to storage slot mapping.
    storage_layout: &'state HashMap<NodeId, StorageSlot>,
    /// The function's declared return types, for `return` to cast to.
    return_types: &'state [Type<'context>],
    /// Whether arithmetic operations use checked variants (`sol.cadd` etc.).
    ///
    /// `true` by default; `false` inside `unchecked {}` blocks.
    checked: bool,
}

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Creates a new statement emitter.
    pub fn new(
        state: &'state Context<'context>,
        environment: &'state mut Environment<'context, 'block>,
        storage_layout: &'state HashMap<NodeId, StorageSlot>,
        return_types: &'state [Type<'context>],
    ) -> Self {
        Self {
            state,
            environment,
            storage_layout,
            return_types,
            checked: true,
        }
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
            Statement::ExpressionStatement(statement) => {
                self.emit_expression_statement(statement, block)
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
            Statement::UncheckedBlock(inner) => self.emit_unchecked_block(inner, block),
            Statement::EmitStatement(emit_statement) => self.emit_event(emit_statement, block),
            Statement::RevertStatement(revert) => self.emit_revert(revert, block),
            _ => unimplemented!(
                "statement lowering: {:?}",
                std::mem::discriminant(statement)
            ),
        }
    }
}
