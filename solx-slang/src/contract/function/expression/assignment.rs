//!
//! Assignment expressions, plain and compound, through the left operand's place.
//!

use slang_solidity_v2::ast::AssignmentExpression;
use slang_solidity_v2::ast::AssignmentExpressionOperator;

use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// `lhs = rhs` and the compound `lhs op= rhs`, lowered through the left operand's place. The
    /// right operand is fully evaluated before the place's old value is read, matching solc's
    /// evaluation order; plain `=` stores the right operand without the read-modify load.
    pub fn assignment(&mut self, node: &AssignmentExpression) -> Value<'context> {
        let (place, element_type) = self.expression_place(&node.left_operand());
        if place.r#type() == element_type {
            unimplemented!(
                "assignment through a reference-typed place in storage or calldata is not yet supported"
            );
        }
        let rhs = self
            .expression(&node.right_operand())
            .coerce(element_type, self);
        let stored =
            match node.operator() {
                AssignmentExpressionOperator::Equal(_) => rhs,
                AssignmentExpressionOperator::PlusEqual(_) => {
                    place.load(element_type, self).add(rhs, self.checked, self)
                }
                AssignmentExpressionOperator::MinusEqual(_) => place
                    .load(element_type, self)
                    .subtract(rhs, self.checked, self),
                AssignmentExpressionOperator::AsteriskEqual(_) => place
                    .load(element_type, self)
                    .multiply(rhs, self.checked, self),
                AssignmentExpressionOperator::SlashEqual(_) => place
                    .load(element_type, self)
                    .divide(rhs, self.checked, self),
                AssignmentExpressionOperator::PercentEqual(_) => {
                    place.load(element_type, self).remainder(rhs, self)
                }
                AssignmentExpressionOperator::AmpersandEqual(_) => {
                    place.load(element_type, self).bitand(rhs, self)
                }
                AssignmentExpressionOperator::BarEqual(_) => {
                    place.load(element_type, self).bitor(rhs, self)
                }
                AssignmentExpressionOperator::CaretEqual(_) => {
                    place.load(element_type, self).bitxor(rhs, self)
                }
                AssignmentExpressionOperator::LessThanLessThanEqual(_) => {
                    place.load(element_type, self).shl(rhs, self)
                }
                AssignmentExpressionOperator::GreaterThanGreaterThanEqual(_)
                | AssignmentExpressionOperator::GreaterThanGreaterThanGreaterThanEqual(_) => {
                    place.load(element_type, self).shr(rhs, self)
                }
            };
        place.store(stored, self);
        stored
    }
}
