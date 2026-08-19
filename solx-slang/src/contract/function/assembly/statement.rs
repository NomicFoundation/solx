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

use crate::scope::assembly::AssemblyScope;

impl<'function, 'contract, 'source_unit, 'context>
    AssemblyScope<'function, 'contract, 'source_unit, 'context>
{
    /// A Yul statement sequence, stopping after a terminator: `break`, `continue`, and `leave`
    /// terminate their block, and Yul permits unreachable statements after one.
    ///
    /// A nested `{ .. }` opens no MLIR region: Yul's block scoping is already resolved into the
    /// per-declaration bindings the scope holds.
    pub fn yul_statements(&mut self, node: &YulBlock) {
        for statement in node.statements().iter() {
            self.yul_statement(&statement);
            if self.current_block().is_terminated() {
                break;
            }
        }
    }

    /// A Yul statement, routed to its lowering. A function definition is emitted ahead of the body
    /// it appears in, so reaching one here is a no-op.
    fn yul_statement(&mut self, node: &YulStatement) {
        match node {
            YulStatement::YulBlock(inner) => self.yul_statements(inner),
            YulStatement::YulVariableDeclarationStatement(inner) => {
                self.yul_variable_declaration(inner)
            }
            YulStatement::YulVariableAssignmentStatement(inner) => self.yul_assignment(inner),
            YulStatement::YulExpression(inner) => self.yul_expression_effect(inner),
            YulStatement::YulIfStatement(inner) => self.yul_if(inner),
            YulStatement::YulForStatement(inner) => self.yul_for(inner),
            YulStatement::YulSwitchStatement(inner) => self.yul_switch(inner),
            YulStatement::YulBreakStatement(_) => self.current_block().r#break(self),
            YulStatement::YulContinueStatement(_) => self.current_block().r#continue(self),
            YulStatement::YulLeaveStatement(_) => self.func_return(),
            YulStatement::YulFunctionDefinition(_) => {}
        }
    }

    /// A `let` declaration. Every value is materialized before any slot is allocated, and an absent
    /// initializer defaults each variable to zero.
    fn yul_variable_declaration(&mut self, node: &YulVariableDeclarationStatement) {
        let names: Vec<_> = node.variables().iter().collect();
        let values = match node.value() {
            Some(value) => self.yul_expression_words(&value.expression()),
            None => names.iter().map(|_| self.word_zero()).collect(),
        };
        assert_eq!(
            names.len(),
            values.len(),
            "slang validates the arity of a Yul declaration"
        );
        for (name, value) in names.iter().zip(values) {
            self.bind(name.node_id(), value);
        }
    }

    /// An assignment to one or more Yul paths.
    fn yul_assignment(&mut self, node: &YulVariableAssignmentStatement) {
        let paths: Vec<_> = node.variables().iter().collect();
        let values = self.yul_expression_words(&node.expression());
        assert_eq!(
            paths.len(),
            values.len(),
            "slang validates the arity of a Yul assignment"
        );
        for (path, value) in paths.iter().zip(values) {
            let reference = self.yul_reference(path);
            reference.write(value, self);
        }
    }

    /// A Yul `if`, which has no `else`.
    fn yul_if(&mut self, node: &YulIfStatement) {
        let condition = self.yul_expression(&node.condition());
        let body = self.current_block().branch(condition, self);
        let block = node.body();
        self.region(body, |scope| scope.yul_statements(&block));
    }

    /// A Yul `for`. Its initializer declares into the enclosing block, which is why a variable bound
    /// there outlives the loop op in MLIR while Yul scopes it to the loop.
    fn yul_for(&mut self, node: &YulForStatement) {
        self.yul_statements(&node.initialization());
        let (condition_block, body_block, step_block) = self.current_block().for_loop(self);

        let condition = node.condition();
        self.region(condition_block, |scope| {
            let value = scope.yul_expression(&condition);
            scope.current_block().condition(value, scope);
        });
        let body = node.body();
        self.region(body_block, |scope| scope.yul_statements(&body));
        let step = node.iterator();
        self.region(step_block, |scope| scope.yul_statements(&step));
    }

    /// A Yul `switch`. With no case values there is nothing to dispatch on, so the argument is
    /// evaluated for its effects and the default body emitted inline.
    fn yul_switch(&mut self, node: &YulSwitchStatement) {
        let argument = self.yul_expression(&node.expression());
        let cases: Vec<_> = node.value_cases().iter().collect();
        let default = node.default_case();

        if cases.is_empty() {
            if let Some(default) = default {
                self.yul_statements(&default.body());
            }
            return;
        }

        let values: Vec<U256> = cases
            .iter()
            .map(|case| case.value().integer_value())
            .collect();
        let (default_block, case_blocks) = self.current_block().switch(argument, &values, self);

        match default {
            Some(default) => {
                let body = default.body();
                self.region(default_block, |scope| scope.yul_statements(&body));
            }
            None => default_block.r#yield(self),
        }
        for (case, block) in cases.iter().zip(case_blocks) {
            let body = case.body();
            self.region(block, |scope| scope.yul_statements(&body));
        }
    }
}
