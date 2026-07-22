//!
//! Literal expressions: integer numbers, the boolean keywords, and strings.
//!

use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::StringExpression;

use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// The `true`/`false` keyword literals.
    pub fn boolean_literal(&mut self, value: bool) -> Value<'context> {
        Value::boolean(value, self)
    }

    /// A decimal or hexadecimal integer literal, folded to a constant of the binder's type. The two
    /// literal kinds share this lowering; the enum is matched only to reach the concrete node whose
    /// `integer_value` and `get_type` read different terminals.
    pub fn number_literal(&mut self, node: &Expression) -> Value<'context> {
        let (value, slang_type) = match node {
            Expression::DecimalNumberExpression(number) => {
                (number.integer_value(), number.get_type())
            }
            Expression::HexNumberExpression(number) => (number.integer_value(), number.get_type()),
            _ => unreachable!("only decimal and hexadecimal literals lower as number literals"),
        };
        Value::constant_from_bigint(
            &value.expect("an integer literal evaluates to an integer"),
            self.typing(slang_type),
            self,
        )
    }

    /// A string literal, lowered to its Sol dialect string value.
    pub fn string_literal(&mut self, node: &StringExpression) -> Value<'context> {
        Value::string_literal(&node.value(), self)
    }
}
