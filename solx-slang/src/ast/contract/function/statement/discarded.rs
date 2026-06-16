//!
//! An expression in statement position — its value discarded.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::PrefixExpressionOperator;

use crate::ast::Emit;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::assignment::AssignmentTarget;

/// An expression emitted for its side effects, its value discarded — an
/// expression statement (`f();`) or a for-loop step (`i++`).
///
/// The two value-less producers — a void call and `delete` — never reach value
/// position ([`Emit::emit`] on an expression always yields a
/// [`Value`](crate::ast::Value)), so they emit here, the single statement-position
/// node, rather than through value emission.
pub struct Discarded<'expression>(pub &'expression Expression);

impl<'expression, 'state, 'context, 'block, 'scope> Emit<'context, 'block, 'state, 'scope>
    for Discarded<'expression>
where
    'context: 'block,
    'context: 'state,
    'block: 'state,
    'state: 'scope,
{
    type Context = &'scope ExpressionContext<'state, 'context, 'block>;
    type Output = BlockRef<'context, 'block>;

    fn emit(&self, context: Self::Context, block: BlockRef<'context, 'block>) -> Self::Output {
        match self.0 {
            Expression::FunctionCallExpression(call) => context.emit_function_call(call, block).1,
            Expression::PrefixExpression(prefix)
                if matches!(
                    prefix.operator(),
                    PrefixExpressionOperator::DeleteKeyword(_)
                ) =>
            {
                AssignmentTarget::delete(context, &prefix.operand(), block)
            }
            _ => self.0.emit(context, block).block,
        }
    }
}
