//!
//! The ternary conditional operator and the value-branch lowering it anchors.
//!

use solx_mlir::Place;
use solx_mlir::Type;
use solx_mlir::Value;

use crate::contract::function::expression::Expression;
use crate::scope::FunctionScope;

codegen!(
    ConditionalExpression {
        /// The ternary conditional operator: neither operand short-circuits, so there is no
        /// initializer and both arms store their evaluated operand.
        -> Value |node, scope| {
            let result_type = codegen!(@result_type ConditionalExpression, node, scope);
            let condition = Expression::emit(&node.operand(), scope).is_nonzero(scope);
            Self::branch_value(
                scope,
                condition,
                result_type,
                |_context| None,
                |scope| Some(Expression::emit(&node.true_expression(), scope)),
                |scope| Some(Expression::emit(&node.false_expression(), scope)),
            )
        }

        /// Branches `condition` into one `result_type` pointer and loads the merge. The pointer is
        /// first stored with whatever `initializer` yields, then each arm that yields a value
        /// stores it, coerced to `result_type`, while an arm that yields none leaves the
        /// initializing value in place. The shared lowering of `?:` (no initializer, both arms
        /// store) and the short-circuit `&&` / `||` (initialized, the short-circuiting arm empty).
        pub fn branch_value<'context>(
            scope: &mut FunctionScope<'_, '_, 'context>,
            condition: Value<'context>,
            result_type: Type<'context>,
            initializer: impl FnOnce(&mut FunctionScope<'_, '_, 'context>) -> Option<Value<'context>>,
            then: impl FnOnce(&mut FunctionScope<'_, '_, 'context>) -> Option<Value<'context>>,
            r#else: impl FnOnce(&mut FunctionScope<'_, '_, 'context>) -> Option<Value<'context>>,
        ) -> Value<'context> {
            let pointer = Place::stack(result_type, scope);
            if let Some(value) = initializer(scope) {
                pointer.store(value, scope);
            }
            let (then_block, else_block) =
                scope.current_block().branch_with_else(condition, scope);
            scope.region(then_block, |scope| {
                if let Some(value) = then(scope) {
                    pointer.store(value.coerce(result_type, scope), scope);
                }
            });
            scope.region(else_block, |scope| {
                if let Some(value) = r#else(scope) {
                    pointer.store(value.coerce(result_type, scope), scope);
                }
            });
            pointer.load(result_type, scope)
        }
    }
);
