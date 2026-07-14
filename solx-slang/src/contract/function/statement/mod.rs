//!
//! Statement lowering to MLIR operations, routed to each statement kind's lowering.
//!

pub mod block;
pub mod control_flow;
pub mod event;
pub mod expression;
pub mod revert;
pub mod variable_declaration;

use slang_solidity_v2::ast::Statement as SlangStatement;

use self::block::Block;
use self::block::UncheckedBlock;
use self::control_flow::BreakStatement;
use self::control_flow::ContinueStatement;
use self::control_flow::DoWhileStatement;
use self::control_flow::ForStatement;
use self::control_flow::IfStatement;
use self::control_flow::ReturnStatement;
use self::control_flow::WhileStatement;
use self::event::EmitStatement;
use self::expression::ExpressionStatement;
use self::revert::RevertStatement;
use self::variable_declaration::VariableDeclarationStatement;

dispatch!(
    /// Dispatches a statement to its variant's lowering for its effects on the block and
    /// environment.
    Statement(Statement) -> Effect |node, scope| {
        VariableDeclarationStatement,
        ExpressionStatement,
        ReturnStatement,
        IfStatement,
        ForStatement,
        WhileStatement,
        DoWhileStatement,
        BreakStatement,
        ContinueStatement,
        Block,
        UncheckedBlock,
        RevertStatement,
        EmitStatement,
    } else {
        SlangStatement::TryStatement(_) | SlangStatement::AssemblyStatement(_) => {
            unimplemented!("try/assembly statements are not yet supported")
        }
    }
);
