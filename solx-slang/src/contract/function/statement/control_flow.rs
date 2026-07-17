//!
//! Control-flow statements: branches, loops, and the jumps that leave them.
//!

use slang_solidity_v2::ast::BreakStatement;
use slang_solidity_v2::ast::ContinueStatement;
use slang_solidity_v2::ast::DoWhileStatement;
use slang_solidity_v2::ast::ForStatement;
use slang_solidity_v2::ast::ForStatementCondition;
use slang_solidity_v2::ast::ForStatementInitialization;
use slang_solidity_v2::ast::IfStatement;
use slang_solidity_v2::ast::ReturnStatement;
use slang_solidity_v2::ast::WhileStatement;

use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// The `if`/`else` statement.
    pub fn if_statement(&mut self, node: &IfStatement) {
        let condition = self.expression(&node.condition()).is_nonzero(self);
        let then_block = match node.else_branch() {
            Some(else_statement) => {
                let (then_block, else_block) =
                    self.current_block().branch_with_else(condition, self);
                self.region(else_block, |scope| scope.statement(&else_statement));
                then_block
            }
            None => self.current_block().branch(condition, self),
        };
        self.region(then_block, |scope| scope.statement(&node.body()));
    }

    /// The `while` statement.
    pub fn while_statement(&mut self, node: &WhileStatement) {
        let (condition_block, body_block) = self.current_block().while_loop(self);
        self.condition_region(condition_block, |scope| scope.expression(&node.condition()));
        self.region(body_block, |scope| scope.statement(&node.body()));
    }

    /// The `do`/`while` statement.
    pub fn do_while_statement(&mut self, node: &DoWhileStatement) {
        let (body_block, condition_block) = self.current_block().do_while(self);
        self.region(body_block, |scope| scope.statement(&node.body()));
        self.condition_region(condition_block, |scope| scope.expression(&node.condition()));
    }

    /// The `for` statement. The step expression emits unchecked, matching the solc lowering this
    /// pipeline is verified against.
    pub fn for_statement(&mut self, node: &ForStatement) {
        self.nested(|scope| {
            scope.for_statement_initialization(&node.initialization());
            let (condition_block, body_block, step_block) = scope.current_block().for_loop(scope);
            scope.condition_region(condition_block, |scope| match node.condition() {
                ForStatementCondition::ExpressionStatement(statement) => {
                    scope.expression(&statement.expression())
                }
                ForStatementCondition::Semicolon(_) => Value::boolean(true, scope),
            });
            scope.region(body_block, |scope| scope.statement(&node.body()));
            scope.region(step_block, |scope| {
                if let Some(iterator) = node.iterator() {
                    scope.unchecked(|scope| scope.expression_effect(&iterator));
                }
            });
        });
    }

    /// The `for` statement's initialization clause.
    pub fn for_statement_initialization(&mut self, node: &ForStatementInitialization) {
        match node {
            ForStatementInitialization::VariableDeclarationStatement(inner) => {
                self.variable_declaration_statement(inner)
            }
            ForStatementInitialization::ExpressionStatement(inner) => {
                self.expression_effect(&inner.expression())
            }
            ForStatementInitialization::Semicolon(_) => {}
        }
    }

    /// The `break` statement.
    pub fn break_statement(&mut self, _node: &BreakStatement) {
        self.current_block().r#break(self);
    }

    /// The `continue` statement.
    pub fn continue_statement(&mut self, _node: &ContinueStatement) {
        self.current_block().r#continue(self);
    }

    /// The `return` statement, its values coerced to the function's declared return types.
    pub fn return_statement(&mut self, node: &ReturnStatement) {
        let Some(expression) = node.expression() else {
            self.current_block().r#return(&[], self);
            return;
        };
        let targets: Vec<_> = self.return_types.iter().copied().map(Some).collect();
        let values = self.coerced_values(&expression, &targets);
        self.current_block().r#return(&values, self);
    }
}
