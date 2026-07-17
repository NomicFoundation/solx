//!
//! The contextual keyword expressions: `this` today; `super` and `payable` once they lower.
//!

use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// The `this` keyword: the enclosing contract as a value.
    pub fn this_value(&mut self) -> Value<'context> {
        Value::this(
            self.current_contract_type
                .expect("sol.this emitted outside a contract"),
            self,
        )
    }
}
