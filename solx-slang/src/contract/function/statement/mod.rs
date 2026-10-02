//!
//! Statement lowering to MLIR operations, routed to each statement kind's lowering.
//!

pub mod block;
pub mod control_flow;
pub mod event;
pub mod revert;
pub mod try_statement;
pub mod variable_declaration;

use slang_solidity_v2::ast::Statement;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// Lowers a statement for its effects on the current block and environment, routing each kind to
    /// its lowering. The ops a statement emits carry its location, except an expression statement
    /// and a block, checked or unchecked, which emit no op of their own: each belongs to a node
    /// within.
    pub fn statement(&mut self, node: &Statement) {
        match node {
            Statement::VariableDeclarationStatement(inner) => {
                self.at_node(inner, |scope| scope.variable_declaration_statement(inner))
            }
            Statement::ExpressionStatement(inner) => self.expression_effect(&inner.expression()),
            Statement::ReturnStatement(inner) => {
                self.at_node(inner, |scope| scope.return_statement(inner))
            }
            Statement::IfStatement(inner) => self.at_node(inner, |scope| scope.if_statement(inner)),
            Statement::ForStatement(inner) => {
                self.at_node(inner, |scope| scope.for_statement(inner))
            }
            Statement::WhileStatement(inner) => {
                self.at_node(inner, |scope| scope.while_statement(inner))
            }
            Statement::DoWhileStatement(inner) => {
                self.at_node(inner, |scope| scope.do_while_statement(inner))
            }
            Statement::BreakStatement(inner) => {
                self.at_node(inner, |scope| scope.break_statement(inner))
            }
            Statement::ContinueStatement(inner) => {
                self.at_node(inner, |scope| scope.continue_statement(inner))
            }
            Statement::Block(inner) => self.block(inner),
            Statement::UncheckedBlock(inner) => self.unchecked_block(inner),
            Statement::RevertStatement(inner) => {
                self.at_node(inner, |scope| scope.revert_statement(inner))
            }
            Statement::EmitStatement(inner) => {
                self.at_node(inner, |scope| scope.emit_statement(inner))
            }
            Statement::TryStatement(inner) => {
                self.at_node(inner, |scope| scope.try_statement(inner))
            }
            Statement::AssemblyStatement(inner) => {
                self.at_node(inner, |scope| scope.assembly_statement(inner))
            }
        }
    }
}
