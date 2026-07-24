//!
//! Assignment expressions, plain and compound, through the left operand's place.
//!

use slang_solidity_v2::ast::AssignmentExpression;
use slang_solidity_v2::ast::AssignmentExpressionOperator;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::Type;

use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// `lhs = rhs` and the compound `lhs op= rhs`. A multi-element tuple destructures and yields
    /// nothing, tuple assignment being value-less; every other form yields the stored value. The right
    /// operand is evaluated before the left place, matching legacy's right-before-left order; a tuple
    /// evaluates all values then all places left-first and commits the stores right-first. Whether the
    /// place is a reference (its own address, taking a `sol.copy` that bridges type and data location)
    /// or a scalar slot (a `store` of the value converted to the slot's own type) is read from the
    /// resolved place. The lone exception is a string or hex literal at a bytes-like target: it folds
    /// to a constant that needs the slot type up front and carries no side effect, so its place is
    /// resolved first. A shift compound keeps its right operand at its own type, every other converts it.
    pub fn assignment(&mut self, node: &AssignmentExpression) -> Option<Value<'context>> {
        let left = node.left_operand();
        let right = node.right_operand();
        let left_type = left.get_type();

        if let Some(Type::Tuple(_)) = &left_type {
            let values = self.expression_values(&right);
            let places = self.expression_places(&left);
            for (place, value) in places.into_iter().zip(values).rev() {
                if let Some((place, r#type)) = place {
                    place.assign(value, r#type, self);
                }
            }
            return None;
        }

        let element_type = self.typing(left_type);

        if let AssignmentExpressionOperator::Equal(_) = node.operator() {
            if let Expression::StringExpression(_) = &right
                && element_type.is_bytes_like()
            {
                let (place, r#type) = self.expression_place(&left);
                let value = self.converted(&right, r#type);
                return Some(place.assign(value, r#type, self));
            }
            let value = self.expression(&right);
            let (place, r#type) = self.expression_place(&left);
            return Some(place.assign(value, r#type, self));
        }

        let rhs = match node.operator() {
            AssignmentExpressionOperator::LessThanLessThanEqual(_)
            | AssignmentExpressionOperator::GreaterThanGreaterThanEqual(_) => {
                self.expression(&right)
            }
            _ => self.converted(&right, element_type),
        };
        let (place, r#type) = self.expression_place(&left);
        let current = place.load(r#type, self);
        let assigned = match node.operator() {
            AssignmentExpressionOperator::PlusEqual(_) => current.add(rhs, self.checked, self),
            AssignmentExpressionOperator::MinusEqual(_) => {
                current.subtract(rhs, self.checked, self)
            }
            AssignmentExpressionOperator::AsteriskEqual(_) => {
                current.multiply(rhs, self.checked, self)
            }
            AssignmentExpressionOperator::SlashEqual(_) => current.divide(rhs, self.checked, self),
            AssignmentExpressionOperator::PercentEqual(_) => current.remainder(rhs, self),
            AssignmentExpressionOperator::AmpersandEqual(_) => current.bitand(rhs, self),
            AssignmentExpressionOperator::BarEqual(_) => current.bitor(rhs, self),
            AssignmentExpressionOperator::CaretEqual(_) => current.bitxor(rhs, self),
            AssignmentExpressionOperator::LessThanLessThanEqual(_) => current.shl(rhs, self),
            AssignmentExpressionOperator::GreaterThanGreaterThanEqual(_) => current.shr(rhs, self),
            AssignmentExpressionOperator::Equal(_)
            | AssignmentExpressionOperator::GreaterThanGreaterThanGreaterThanEqual(_) => {
                unreachable!("`=` is handled above and Solidity has no `>>>`")
            }
        };
        place.store(assigned, self);
        Some(assigned)
    }
}
