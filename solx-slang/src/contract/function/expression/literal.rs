//!
//! Literal expressions: the boolean keywords and strings.
//!

use slang_solidity_v2::ast::StringExpression;

use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// The `true`/`false` keyword literals.
    pub fn boolean_literal(&mut self, value: bool) -> Value<'context> {
        Value::boolean(value, self)
    }

    /// A string literal, lowered to its Sol dialect string value.
    pub fn string_literal(&mut self, node: &StringExpression) -> Value<'context> {
        Value::string_literal(&node.value(), self)
    }
}
