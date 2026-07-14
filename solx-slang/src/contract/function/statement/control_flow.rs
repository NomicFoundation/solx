//!
//! Control-flow statements: branches, loops, and the jumps that leave them.
//!

use slang_solidity_v2::ast::ForStatementCondition;

use solx_mlir::Value;

use crate::contract::function::expression::Expression;
use crate::contract::function::statement::Statement;
use crate::contract::function::statement::expression::ExpressionStatement;
use crate::contract::function::statement::variable_declaration::VariableDeclarationStatement;

codegen!(
    /// The `if`/`else` statement.
    IfStatement -> Effect |node, scope| {
        let condition = Expression::emit(&node.condition(), scope).is_nonzero(scope);
        let then_block = match node.else_branch() {
            Some(else_statement) => {
                let (then_block, else_block) =
                    scope.current_block().branch_with_else(condition, scope);
                scope.region(else_block, |scope| Statement::emit(&else_statement, scope));
                then_block
            }
            None => scope.current_block().branch(condition, scope),
        };
        scope.region(then_block, |scope| Statement::emit(&node.body(), scope));
    }

    /// The `while` statement.
    WhileStatement -> Effect |node, scope| {
        let (condition_block, body_block) = scope.current_block().while_loop(scope);
        scope.condition_region(condition_block, |scope| {
            Expression::emit(&node.condition(), scope)
        });
        scope.region(body_block, |scope| Statement::emit(&node.body(), scope));
    }

    /// The `do`/`while` statement.
    DoWhileStatement -> Effect |node, scope| {
        let (body_block, condition_block) = scope.current_block().do_while(scope);
        scope.region(body_block, |scope| Statement::emit(&node.body(), scope));
        scope.condition_region(condition_block, |scope| {
            Expression::emit(&node.condition(), scope)
        });
    }

    /// The `for` statement. The step expression emits unchecked, matching the solc lowering this
    /// pipeline is verified against.
    ForStatement -> Effect |node, scope| {
        scope.nested(|scope| {
            ForStatementInitialization::emit(&node.initialization(), scope);
            let (condition_block, body_block, step_block) =
                scope.current_block().for_loop(scope);
            scope.condition_region(condition_block, |scope| match node.condition() {
                ForStatementCondition::ExpressionStatement(statement) => {
                    Expression::emit(&statement.expression(), scope)
                }
                ForStatementCondition::Semicolon(_) => Value::boolean(true, scope),
            });
            scope.region(body_block, |scope| Statement::emit(&node.body(), scope));
            scope.region(step_block, |scope| {
                if let Some(iterator) = node.iterator() {
                    scope.unchecked(|scope| Expression::emit_for_effect(&iterator, scope));
                }
            });
        });
    }

    BreakStatement -> Effect |_node, scope| {
        scope.current_block().r#break(scope);
    }

    ContinueStatement -> Effect |_node, scope| {
        scope.current_block().r#continue(scope);
    }

    /// The `return` statement, its values coerced to the function's declared return types.
    ReturnStatement -> Effect |node, scope| {
        let Some(expression) = node.expression() else {
            scope.current_block().r#return(&[], scope);
            return;
        };
        let values = if scope.return_types().len() > 1 {
            Expression::emit_values(&expression, scope)
        } else {
            vec![Expression::emit(&expression, scope)]
        };
        let values: Vec<_> = values
            .iter()
            .zip(scope.return_types())
            .map(|(value, &return_type)| value.coerce(return_type, scope))
            .collect();
        scope.current_block().r#return(&values, scope);
    }
);

dispatch!(
    /// The `for` statement's initialization clause.
    ForStatementInitialization(ForStatementInitialization) -> Effect |node, scope| {
        VariableDeclarationStatement,
        ExpressionStatement,
    } else {
        ::slang_solidity_v2::ast::ForStatementInitialization::Semicolon(_) => {}
    }
);
