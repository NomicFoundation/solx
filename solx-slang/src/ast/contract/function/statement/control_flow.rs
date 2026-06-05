//!
//! Control flow statement lowering: `if`/`else`, `for`, `while`, `do`/`while`,
//! `break`, `continue`, and nested (including `unchecked`) blocks.
//!

use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::RegionLike;
use slang_solidity_v2::ast::DoWhileStatement;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::ForStatement;
use slang_solidity_v2::ast::ForStatementCondition;
use slang_solidity_v2::ast::ForStatementInitialization;
use slang_solidity_v2::ast::IfStatement;
use slang_solidity_v2::ast::Statement;
use slang_solidity_v2::ast::Statements;
use slang_solidity_v2::ast::UncheckedBlock;
use slang_solidity_v2::ast::WhileStatement;

use crate::ast::contract::function::expression::ExpressionEmitter;

use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Lowers an `if`/`else` statement to `sol.if`.
    pub fn emit_if(
        &mut self,
        if_statement: &IfStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let condition = if_statement.condition();
        let (condition, block) = {
            let emitter = ExpressionEmitter::new(
                self.state,
                self.environment,
                self.storage_layout,
                self.checked,
            );
            let (value, block) = emitter.emit_value(&condition, block)?;
            (emitter.emit_is_nonzero(value, &block), block)
        };

        let (then_block, else_block) = self.state.builder.emit_sol_if(condition, &block);
        self.emit_branch(&if_statement.body(), then_block)?;
        match if_statement.else_branch() {
            Some(else_statement) => self.emit_branch(&else_statement, else_block)?,
            None => self.state.builder.emit_sol_yield(&else_block),
        }
        Ok(Some(block))
    }

    /// Lowers a `while` loop to `sol.while`.
    pub fn emit_while(
        &mut self,
        while_statement: &WhileStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let (condition_block, body_block) = self.state.builder.emit_sol_while(&block);
        self.emit_loop_condition(&while_statement.condition(), condition_block)?;
        self.emit_loop_body(&while_statement.body(), body_block)?;
        Ok(Some(block))
    }

    /// Lowers a `do`/`while` loop to `sol.do`; the body runs before the
    /// condition is first tested.
    pub fn emit_do_while(
        &mut self,
        do_while: &DoWhileStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let (body_block, condition_block) = self.state.builder.emit_sol_do_while(&block);
        self.emit_loop_body(&do_while.body(), body_block)?;
        self.emit_loop_condition(&do_while.condition(), condition_block)?;
        Ok(Some(block))
    }

    /// Lowers a `for` loop to `sol.for`. The initializer runs in the current
    /// block; a fresh lexical scope covers it and the body.
    pub fn emit_for(
        &mut self,
        for_statement: &ForStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        self.environment.enter_scope();
        let block = match self.emit_for_initialization(for_statement, block)? {
            Some(block) => block,
            None => {
                self.environment.exit_scope();
                return Ok(None);
            }
        };

        let (condition_block, body_block, step_block) = self.state.builder.emit_sol_for(&block);
        self.emit_for_condition(for_statement, condition_block)?;
        self.emit_loop_body(&for_statement.body(), body_block)?;
        self.emit_for_step(for_statement, step_block)?;

        self.environment.exit_scope();
        Ok(Some(block))
    }

    /// Emits a `sol.break` terminator, ending the current control flow.
    pub fn emit_break(
        &self,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        self.state.builder.emit_sol_break(&block);
        Ok(None)
    }

    /// Emits a `sol.continue` terminator, ending the current control flow.
    pub fn emit_continue(
        &self,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        self.state.builder.emit_sol_continue(&block);
        Ok(None)
    }

    /// Lowers a nested block in its own lexical scope.
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

    /// Lowers an `unchecked { … }` block, suppressing overflow checks for the
    /// arithmetic inside it.
    pub fn emit_unchecked_block(
        &mut self,
        unchecked: &UncheckedBlock,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let outer_checked = self.checked;
        self.checked = false;
        let result = self.emit_block(unchecked.block().statements(), block);
        self.checked = outer_checked;
        result
    }

    /// Emits an `if` branch body into `block`, terminating its region with
    /// `sol.yield` — a fresh dead block when the body already terminated (e.g.
    /// via `return`), matching solc's always-yield region shape.
    fn emit_branch(
        &mut self,
        body: &Statement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        let region = block
            .parent_region()
            .expect("sol.if branch block belongs to a region");
        match self.emit(body, block)? {
            Some(end) => self.state.builder.emit_sol_yield(&end),
            None => self.emit_dead_yield(&region),
        }
        Ok(())
    }

    /// Emits a loop condition into its region, terminating with `sol.condition`.
    fn emit_loop_condition(
        &self,
        condition: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
        let (value, block) = emitter.emit_value(condition, block)?;
        let condition = emitter.emit_is_nonzero(value, &block);
        self.state.builder.emit_sol_condition(condition, &block);
        Ok(())
    }

    /// Emits a loop body into its region, terminating with `sol.yield` unless
    /// the body already terminated (via `break`, `continue`, or `return`).
    fn emit_loop_body(
        &mut self,
        body: &Statement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        if let Some(end) = self.emit(body, block)? {
            self.state.builder.emit_sol_yield(&end);
        }
        Ok(())
    }

    /// Emits a `for` initializer (`T i = …`, an expression, or empty).
    fn emit_for_initialization(
        &mut self,
        for_statement: &ForStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        match for_statement.initialization() {
            ForStatementInitialization::VariableDeclarationStatement(declaration) => {
                self.emit(&Statement::VariableDeclarationStatement(declaration), block)
            }
            ForStatementInitialization::ExpressionStatement(statement) => {
                self.emit(&Statement::ExpressionStatement(statement), block)
            }
            ForStatementInitialization::Semicolon(_) => Ok(Some(block)),
        }
    }

    /// Emits a `for` condition (an expression, or `true` when omitted).
    fn emit_for_condition(
        &self,
        for_statement: &ForStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        match for_statement.condition() {
            ForStatementCondition::ExpressionStatement(statement) => {
                self.emit_loop_condition(&statement.expression(), block)
            }
            ForStatementCondition::Semicolon(_) => {
                let always = self.state.builder.emit_bool(true, &block);
                self.state.builder.emit_sol_condition(always, &block);
                Ok(())
            }
        }
    }

    /// Emits a `for` step expression (always unchecked, matching solc's `i++`),
    /// terminating its region with `sol.yield`.
    fn emit_for_step(
        &self,
        for_statement: &ForStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        let block = match for_statement.iterator() {
            Some(iterator) => {
                let emitter = ExpressionEmitter::new(
                    self.state,
                    self.environment,
                    self.storage_layout,
                    false,
                );
                let (_value, block) = emitter.emit(&iterator, block)?;
                block
            }
            None => block,
        };
        self.state.builder.emit_sol_yield(&block);
        Ok(())
    }

    /// Appends a dead block terminated by `sol.yield` to a region whose live
    /// block already terminated, satisfying the region's yield requirement.
    pub fn emit_dead_yield(&self, region: &Region<'context>) {
        let dead_block = Block::new(&[]);
        self.state.builder.emit_sol_yield(&dead_block);
        region.append_block(dead_block);
    }
}
