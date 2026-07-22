//!
//! Assignment expressions, plain and compound, through the left operand's place.
//!

use slang_solidity_v2::ast::AssignmentExpression;
use slang_solidity_v2::ast::AssignmentExpressionOperator;

use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// `lhs = rhs` and the compound `lhs op= rhs`. A multi-element tuple left operand destructures and
    /// yields nothing, tuple assignment being value-less; every other form yields the stored value. A
    /// reference-typed place is its own address and takes a `sol.copy` of the right operand, which
    /// bridges both its type and its data location. A shift compound keeps its right operand at its
    /// own type, every other form coerces it to the place's type.
    pub fn assignment(&mut self, node: &AssignmentExpression) -> Option<Value<'context>> {
        let places = self.expression_places(&node.left_operand());
        if places.len() > 1 {
            let targets: Vec<_> = places
                .iter()
                .map(|&element| {
                    let Some((place, element_type)) = element else {
                        return None;
                    };
                    (place.r#type() != element_type).then_some(element_type)
                })
                .collect();
            let values = self.coerced_values(&node.right_operand(), &targets);
            for (place, value) in places.into_iter().zip(values) {
                let Some((place, element_type)) = place else {
                    continue;
                };
                if place.r#type() == element_type {
                    place.copy_from(value, self);
                } else {
                    place.store(value, self);
                }
            }
            return None;
        }

        let (place, element_type) = places
            .into_iter()
            .next()
            .flatten()
            .expect("an assignment target denotes a place");

        if place.r#type() == element_type {
            let source = self.expression(&node.right_operand());
            place.copy_from(source, self);
            return Some(Value::from(place));
        }

        if let AssignmentExpressionOperator::Equal(_) = node.operator() {
            let value = self.coerced(&node.right_operand(), element_type);
            place.store(value, self);
            return Some(value);
        }

        let rhs = match node.operator() {
            AssignmentExpressionOperator::LessThanLessThanEqual(_)
            | AssignmentExpressionOperator::GreaterThanGreaterThanEqual(_) => {
                self.expression(&node.right_operand())
            }
            _ => self.coerced(&node.right_operand(), element_type),
        };
        let current = place.load(element_type, self);
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
