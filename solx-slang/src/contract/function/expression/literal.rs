//!
//! Literal expressions: the boolean keywords, hex numbers and strings.
//!

use slang_solidity_v2::ast::HexNumberExpression;
use slang_solidity_v2::ast::StringExpression;

use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// The `true`/`false` keyword literals.
    pub fn boolean_literal(&mut self, value: bool) -> Value<'context> {
        Value::boolean(value, self)
    }

    /// A 40-digit hex literal, which Slang types `address` and leaves unfolded.
    pub fn hex_number_literal(&mut self, node: &HexNumberExpression) -> Value<'context> {
        let value = node
            .integer_value()
            .expect("an address literal spells its value in hex digits");
        Value::constant_from_bigint(&value, self.typing(node.get_type()), self)
    }

    /// A string literal, lowered to its Sol dialect string value.
    pub fn string_literal(&mut self, node: &StringExpression) -> Value<'context> {
        Value::string_literal(&node.value(), self)
    }
}
