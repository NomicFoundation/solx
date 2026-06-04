//!
//! Assignment expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::AssignmentExpression;
use slang_solidity_v2::ast::AssignmentExpressionOperator;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers a simple assignment (`=`); the cast right-hand side is stored to
    /// the target and is also the expression's result.
    ///
    /// Compound assignments are lowered by a later domain.
    pub fn emit_assignment(
        &self,
        assignment: &AssignmentExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        if !matches!(
            assignment.operator(),
            AssignmentExpressionOperator::Equal(_)
        ) {
            unimplemented!("compound assignment lowering");
        }

        let lvalue = self.resolve_lvalue(&assignment.left_operand());
        let (value, block) = self.emit_value(&assignment.right_operand(), block)?;
        let stored = TypeConversion::from_target_type(lvalue.element_type(), &self.state.builder)
            .emit(value, &self.state.builder, &block);
        self.emit_lvalue_store(&lvalue, stored, &block);
        Ok((stored, block))
    }
}
