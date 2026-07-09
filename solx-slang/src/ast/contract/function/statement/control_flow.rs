//!
//! Control flow statement lowering: if/else, for, while, do-while.
//!

use slang_solidity_v2::ast::ForStatementCondition;
use slang_solidity_v2::ast::ForStatementInitialization;
use slang_solidity_v2::ast::Statement;

use solx_mlir::Context;
use solx_mlir::Value;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context> StatementEmitter<'state, 'context> {
    /// Emits an if/else statement using `sol.if`.
    ///
    /// # Errors
    ///
    /// Returns an error if the condition or body contains unsupported constructs.
    pub fn emit_if(
        &mut self,
        if_statement: &slang_solidity_v2::ast::IfStatement,
        context: &mut Context<'context>,
    ) -> anyhow::Result<()> {
        let condition_expression = if_statement.condition();
        let emitter = ExpressionEmitter::new(self.environment, self.storage_layout, self.checked);
        let condition_value = emitter.emit_value(&condition_expression, context)?;
        let condition_boolean = emitter.emit_is_nonzero(condition_value, context);

        let parent = context.current_block();
        let else_branch = if_statement.else_branch();
        let (then_block, else_block) =
            parent.branch(condition_boolean, else_branch.is_some(), context);

        context.current_block = Some(then_block);
        self.emit(&if_statement.body(), context)?;
        let then_end = context.current_block();
        if !then_end.is_terminated() {
            then_end.r#yield(&[], context);
        }

        if let (Some(else_statement), Some(else_block)) = (else_branch.as_ref(), else_block) {
            context.current_block = Some(else_block);
            self.emit(else_statement, context)?;
            let else_end = context.current_block();
            if !else_end.is_terminated() {
                else_end.r#yield(&[], context);
            }
        }

        context.current_block = Some(parent);
        Ok(())
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
        context: &mut Context<'context>,
    ) -> anyhow::Result<()> {
        self.environment.enter_scope();

        match for_statement.initialization() {
            ForStatementInitialization::VariableDeclarationStatement(declaration) => {
                self.emit(
                    &Statement::VariableDeclarationStatement(declaration.clone()),
                    context,
                )?;
            }
            ForStatementInitialization::ExpressionStatement(expression_statement) => {
                self.emit(
                    &Statement::ExpressionStatement(expression_statement.clone()),
                    context,
                )?;
            }
            ForStatementInitialization::Semicolon(_) => {}
        }

        let parent = context.current_block();
        let (condition_block, body_block, step_block) = parent.for_loop(context);

        context.current_block = Some(condition_block);
        match for_statement.condition() {
            ForStatementCondition::ExpressionStatement(expression_statement) => {
                let expression = expression_statement.expression();
                let emitter =
                    ExpressionEmitter::new(self.environment, self.storage_layout, self.checked);
                let condition_value = emitter.emit_value(&expression, context)?;
                let condition_boolean = emitter.emit_is_nonzero(condition_value, context);
                let condition_end = context.current_block();
                condition_end.condition(condition_boolean, context);
            }
            ForStatementCondition::Semicolon(_) => {
                let true_value = Value::boolean(true, context);
                let condition_block = context.current_block();
                condition_block.condition(true_value, context);
            }
        }

        context.current_block = Some(body_block);
        self.emit(&for_statement.body(), context)?;
        let body_end = context.current_block();
        if !body_end.is_terminated() {
            body_end.r#yield(&[], context);
        }

        context.current_block = Some(step_block);
        if let Some(ref iterator_expression) = for_statement.iterator() {
            let emitter = ExpressionEmitter::new(self.environment, self.storage_layout, false);
            emitter.emit(iterator_expression, context)?;
        }
        let step_end = context.current_block();
        step_end.r#yield(&[], context);

        self.environment.exit_scope();
        context.current_block = Some(parent);
        Ok(())
    }

    /// Emits a while loop using `sol.while`.
    ///
    /// # Errors
    ///
    /// Returns an error if the condition or body contains unsupported constructs.
    pub fn emit_while(
        &mut self,
        while_statement: &slang_solidity_v2::ast::WhileStatement,
        context: &mut Context<'context>,
    ) -> anyhow::Result<()> {
        let parent = context.current_block();
        let (condition_block, body_block) = parent.while_loop(context);

        context.current_block = Some(condition_block);
        let condition_expression = while_statement.condition();
        let emitter = ExpressionEmitter::new(self.environment, self.storage_layout, self.checked);
        let condition_value = emitter.emit_value(&condition_expression, context)?;
        let condition_boolean = emitter.emit_is_nonzero(condition_value, context);
        let condition_end = context.current_block();
        condition_end.condition(condition_boolean, context);

        context.current_block = Some(body_block);
        self.emit(&while_statement.body(), context)?;
        let body_end = context.current_block();
        if !body_end.is_terminated() {
            body_end.r#yield(&[], context);
        }

        context.current_block = Some(parent);
        Ok(())
    }

    /// Emits a do-while loop using `sol.do`.
    ///
    /// # Errors
    ///
    /// Returns an error if the body or condition contains unsupported constructs.
    pub fn emit_do_while(
        &mut self,
        do_while: &slang_solidity_v2::ast::DoWhileStatement,
        context: &mut Context<'context>,
    ) -> anyhow::Result<()> {
        let parent = context.current_block();
        let (body_block, condition_block) = parent.do_while(context);

        context.current_block = Some(body_block);
        self.emit(&do_while.body(), context)?;
        let body_end = context.current_block();
        if !body_end.is_terminated() {
            body_end.r#yield(&[], context);
        }

        context.current_block = Some(condition_block);
        let condition_expression = do_while.condition();
        let emitter = ExpressionEmitter::new(self.environment, self.storage_layout, self.checked);
        let condition_value = emitter.emit_value(&condition_expression, context)?;
        let condition_boolean = emitter.emit_is_nonzero(condition_value, context);
        let condition_end = context.current_block();
        condition_end.condition(condition_boolean, context);

        context.current_block = Some(parent);
        Ok(())
    }
}
