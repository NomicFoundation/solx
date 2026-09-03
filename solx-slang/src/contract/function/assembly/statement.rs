//!
//! Yul statement lowering: declarations, assignments, and the control flow Yul has.
//!

use ruint::aliases::U256;
use slang_solidity_v2::ast::YulBlock;
use slang_solidity_v2::ast::YulForStatement;
use slang_solidity_v2::ast::YulIfStatement;
use slang_solidity_v2::ast::YulStatement;
use slang_solidity_v2::ast::YulSwitchStatement;
use slang_solidity_v2::ast::YulVariableAssignmentStatement;
use slang_solidity_v2::ast::YulVariableDeclarationStatement;

use solx_mlir::Word;

use crate::scope::assembly::AssemblyScope;

impl<'function, 'contract, 'source_unit, 'context>
    AssemblyScope<'function, 'contract, 'source_unit, 'context>
{
    /// A Yul statement sequence, stopping after a terminator: `break`, `continue`, and `leave`
    /// terminate their block, and Yul permits unreachable statements after one.
    ///
    /// A nested `{ .. }` opens no MLIR region: Yul's block scoping is already resolved into the
    /// per-declaration bindings the scope holds.
    pub fn statements(&mut self, node: &YulBlock) {
        for statement in node.statements().iter() {
            self.statement(&statement);
            if self.current_block().is_terminated() {
                break;
            }
        }
    }

    /// A Yul statement, routed to its lowering.
    fn statement(&mut self, node: &YulStatement) {
        match node {
            YulStatement::YulBlock(inner) => self.statements(inner),
            YulStatement::YulVariableDeclarationStatement(inner) => {
                self.variable_declaration_statement(inner)
            }
            YulStatement::YulVariableAssignmentStatement(inner) => {
                self.variable_assignment_statement(inner)
            }
            YulStatement::YulExpression(inner) => self.expression_effect(inner),
            YulStatement::YulIfStatement(inner) => self.if_statement(inner),
            YulStatement::YulForStatement(inner) => self.for_statement(inner),
            YulStatement::YulSwitchStatement(inner) => self.switch_statement(inner),
            YulStatement::YulBreakStatement(_) => self.current_block().r#break(self),
            YulStatement::YulContinueStatement(_) => self.current_block().r#continue(self),
            YulStatement::YulLeaveStatement(_) => self.function_return(),
            YulStatement::YulFunctionDefinition(inner) => {
                self.function_definition(inner);
            }
        }
    }

    /// A `let` declaration, whose absent initializer defaults each variable to zero.
    fn variable_declaration_statement(&mut self, node: &YulVariableDeclarationStatement) {
        let names: Vec<_> = node.variables().iter().collect();
        let values = match node.value() {
            Some(value) => self.expression_words(&value.expression()),
            None => names.iter().map(|_| Word::zero(self)).collect(),
        };
        for (name, value) in names.iter().zip(values) {
            self.bind(name.node_id(), value);
        }
    }

    /// An assignment to one or more Yul paths.
    fn variable_assignment_statement(&mut self, node: &YulVariableAssignmentStatement) {
        let values = self.expression_words(&node.expression());
        for (path, value) in node.variables().iter().zip(values) {
            self.reference(&path).write(value, self);
        }
    }

    /// A Yul `if`, which has no `else`.
    fn if_statement(&mut self, node: &YulIfStatement) {
        let condition = self.expression(&node.condition());
        let body = self.current_block().branch(condition, self);
        self.region(body, |scope| scope.statements(&node.body()));
    }

    /// A Yul `for`. Its initializer declares into the enclosing block, which is why a variable bound
    /// there outlives the loop op in MLIR while Yul scopes it to the loop; a `leave` there
    /// terminates that block and leaves the loop unreachable.
    fn for_statement(&mut self, node: &YulForStatement) {
        self.statements(&node.initialization());
        if self.current_block().is_terminated() {
            return;
        }
        let (condition_block, body_block, step_block) = self.current_block().for_loop(self);

        self.region(condition_block, |scope| {
            let value = scope.expression(&node.condition());
            scope.current_block().condition(value, scope);
        });
        self.region(body_block, |scope| scope.statements(&node.body()));
        self.region(step_block, |scope| scope.statements(&node.iterator()));
    }

    /// A Yul `switch`. With no case values there is nothing to dispatch on, so the argument is
    /// evaluated for its effects and the default body emitted inline.
    fn switch_statement(&mut self, node: &YulSwitchStatement) {
        let argument = self.expression(&node.expression());
        let cases: Vec<_> = node.value_cases().iter().collect();

        if cases.is_empty() {
            let default = node
                .default_case()
                .expect("a Yul switch with no cases has a default");
            self.statements(&default.body());
            return;
        }

        let values: Vec<U256> = cases
            .iter()
            .map(|case| case.value().integer_value())
            .collect();
        let (default_block, case_blocks) = self.current_block().switch(argument, &values, self);

        // Case bodies are emitted before the default, in source order, so a Yul function declared
        // in a case hoists ahead of one declared in the default body.
        for (case, block) in cases.iter().zip(case_blocks) {
            self.region(block, |scope| scope.statements(&case.body()));
        }
        match node.default_case() {
            Some(default) => {
                self.region(default_block, |scope| scope.statements(&default.body()));
            }
            None => default_block.r#yield(self),
        }
    }
}
