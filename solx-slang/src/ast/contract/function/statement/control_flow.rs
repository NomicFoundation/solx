//!
//! Control flow statement lowering: if/else, for, while, do-while.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::ForStatementCondition;
use slang_solidity_v2::ast::ForStatementInitialization;
use slang_solidity_v2::ast::Statement;

use solx_mlir::Effect;
use solx_mlir::Value;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Emits an if/else statement using `sol.if`.
    ///
    /// # Errors
    ///
    /// Returns an error if the condition or body contains unsupported constructs.
    pub fn emit_if(
        &mut self,
        if_statement: &slang_solidity_v2::ast::IfStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let condition_expression = if_statement.condition();
        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
        let (condition_value, block) = emitter.emit_value(&condition_expression, block)?;
        let condition_boolean = emitter.emit_is_nonzero(condition_value, &block);

        let (then_block, else_block) = Effect::new(self.state, block).branch(condition_boolean);

        let then_end = self.emit(&if_statement.body(), then_block)?;
        if let Some(then_end) = then_end {
            Effect::new(self.state, then_end).r#yield(&[]);
        } else {
            Effect::new(self.state, then_block).empty_yield();
        }

        if let Some(ref else_statement) = if_statement.else_branch() {
            let else_end = self.emit(else_statement, else_block)?;
            if let Some(else_end) = else_end {
                Effect::new(self.state, else_end).r#yield(&[]);
            } else {
                Effect::new(self.state, else_block).empty_yield();
            }
        } else {
            Effect::new(self.state, else_block).r#yield(&[]);
        }

        Ok(Some(block))
    }

    /// Emits a for loop using `sol.for`.
    ///
    /// # Errors
    ///
    /// Returns an error if the initialization, condition, body, or step
    /// contains unsupported constructs.
    pub fn emit_for(
        &mut self,
        for_statement: &slang_solidity_v2::ast::ForStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        self.environment.enter_scope();

        let block = match for_statement.initialization() {
            ForStatementInitialization::VariableDeclarationStatement(declaration) => match self
                .emit(
                    &Statement::VariableDeclarationStatement(declaration.clone()),
                    block,
                )? {
                Some(block) => block,
                None => {
                    self.environment.exit_scope();
                    return Ok(None);
                }
            },
            ForStatementInitialization::ExpressionStatement(expression_statement) => match self
                .emit(
                    &Statement::ExpressionStatement(expression_statement.clone()),
                    block,
                )? {
                Some(block) => block,
                None => {
                    self.environment.exit_scope();
                    return Ok(None);
                }
            },
            ForStatementInitialization::Semicolon(_) => block,
        };

        let (condition_block, body_block, step_block) = Effect::new(self.state, block).for_loop();

        match for_statement.condition() {
            ForStatementCondition::ExpressionStatement(expression_statement) => {
                let expression = expression_statement.expression();
                let emitter = ExpressionEmitter::new(
                    self.state,
                    self.environment,
                    self.storage_layout,
                    self.checked,
                );
                let (condition_value, condition_end) =
                    emitter.emit_value(&expression, condition_block)?;
                let condition_boolean = emitter.emit_is_nonzero(condition_value, &condition_end);
                Effect::new(self.state, condition_end).condition(condition_boolean);
            }
            ForStatementCondition::Semicolon(_) => {
                let true_value = Value::boolean(true, self.state, &condition_block);
                Effect::new(self.state, condition_block).condition(true_value);
            }
        }

        let body_end = self.emit(&for_statement.body(), body_block)?;
        if let Some(body_end) = body_end {
            Effect::new(self.state, body_end).r#yield(&[]);
        }

        if let Some(ref iterator_expression) = for_statement.iterator() {
            let emitter =
                ExpressionEmitter::new(self.state, self.environment, self.storage_layout, false);
            let (_, step_end) = emitter.emit(iterator_expression, step_block)?;
            Effect::new(self.state, step_end).r#yield(&[]);
        } else {
            Effect::new(self.state, step_block).r#yield(&[]);
        }

        self.environment.exit_scope();
        Ok(Some(block))
    }

    /// Emits a while loop using `sol.while`.
    ///
    /// # Errors
    ///
    /// Returns an error if the condition or body contains unsupported constructs.
    pub fn emit_while(
        &mut self,
        while_statement: &slang_solidity_v2::ast::WhileStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let (condition_block, body_block) = Effect::new(self.state, block).while_loop();

        let condition_expression = while_statement.condition();
        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
        let (condition_value, condition_end) =
            emitter.emit_value(&condition_expression, condition_block)?;
        let condition_boolean = emitter.emit_is_nonzero(condition_value, &condition_end);
        Effect::new(self.state, condition_end).condition(condition_boolean);

        let body_end = self.emit(&while_statement.body(), body_block)?;
        if let Some(body_end) = body_end {
            Effect::new(self.state, body_end).r#yield(&[]);
        }

        Ok(Some(block))
    }

    /// Emits a do-while loop using `sol.do`.
    ///
    /// # Errors
    ///
    /// Returns an error if the body or condition contains unsupported constructs.
    pub fn emit_do_while(
        &mut self,
        do_while: &slang_solidity_v2::ast::DoWhileStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let (body_block, condition_block) = Effect::new(self.state, block).do_while();

        let body_end = self.emit(&do_while.body(), body_block)?;
        if let Some(body_end) = body_end {
            Effect::new(self.state, body_end).r#yield(&[]);
        }

        let condition_expression = do_while.condition();
        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
        let (condition_value, condition_end) =
            emitter.emit_value(&condition_expression, condition_block)?;
        let condition_boolean = emitter.emit_is_nonzero(condition_value, &condition_end);
        Effect::new(self.state, condition_end).condition(condition_boolean);

        Ok(Some(block))
    }
}
