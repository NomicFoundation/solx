//!
//! Control flow statement lowering: `if`/`else`, `for`, `while`, `do`/`while`,
//! `break`, `continue`, and nested (including `unchecked`) blocks.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::DoWhileStatement;
use slang_solidity_v2::ast::ForStatement;
use slang_solidity_v2::ast::IfStatement;
use slang_solidity_v2::ast::Statements;
use slang_solidity_v2::ast::UncheckedBlock;
use slang_solidity_v2::ast::WhileStatement;

use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Lowers an `if`/`else` statement to `sol.if`.
    pub fn emit_if(
        &mut self,
        _if_statement: &IfStatement,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        unimplemented!("control flow: if")
    }

    /// Lowers a `while` loop to `sol.while`.
    pub fn emit_while(
        &mut self,
        _while_statement: &WhileStatement,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        unimplemented!("control flow: while")
    }

    /// Lowers a `do`/`while` loop.
    pub fn emit_do_while(
        &mut self,
        _do_while: &DoWhileStatement,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        unimplemented!("control flow: do/while")
    }

    /// Lowers a `for` loop.
    pub fn emit_for(
        &mut self,
        _for_statement: &ForStatement,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        unimplemented!("control flow: for")
    }

    /// Lowers a `break` statement.
    pub fn emit_break(
        &self,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        unimplemented!("control flow: break")
    }

    /// Lowers a `continue` statement.
    pub fn emit_continue(
        &self,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        unimplemented!("control flow: continue")
    }

    /// Lowers a nested block, opening a new variable scope.
    pub fn emit_block(
        &mut self,
        _statements: Statements,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        unimplemented!("control flow: block")
    }

    /// Lowers an `unchecked { … }` block.
    pub fn emit_unchecked_block(
        &mut self,
        _unchecked: &UncheckedBlock,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        unimplemented!("control flow: unchecked block")
    }
}
