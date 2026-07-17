//!
//! Statement lowering to MLIR operations, routed to each statement kind's lowering.
//!

pub mod block;
pub mod control_flow;
pub mod event;
pub mod revert;
pub mod variable_declaration;

use slang_solidity_v2::ast::Statement;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// Lowers a statement for its effects on the current block and environment, routing each kind to
    /// its lowering.
    pub fn statement(&mut self, node: &Statement) {
        match node {
            Statement::VariableDeclarationStatement(inner) => {
                self.variable_declaration_statement(inner)
            }
            Statement::ExpressionStatement(inner) => self.expression_effect(&inner.expression()),
            Statement::ReturnStatement(inner) => self.return_statement(inner),
            Statement::IfStatement(inner) => self.if_statement(inner),
            Statement::ForStatement(inner) => self.for_statement(inner),
            Statement::WhileStatement(inner) => self.while_statement(inner),
            Statement::DoWhileStatement(inner) => self.do_while_statement(inner),
            Statement::BreakStatement(inner) => self.break_statement(inner),
            Statement::ContinueStatement(inner) => self.continue_statement(inner),
            Statement::Block(inner) => self.block(inner),
            Statement::UncheckedBlock(inner) => self.unchecked_block(inner),
            Statement::RevertStatement(inner) => self.revert_statement(inner),
            Statement::EmitStatement(inner) => self.emit_statement(inner),
            Statement::TryStatement(_) | Statement::AssemblyStatement(_) => {
                unimplemented!("try/assembly statements are not yet supported")
            }
        }
    }
}
