//!
//! Array literal expressions.
//!

use slang_solidity_v2::ast::ArrayExpression;
use slang_solidity_v2::ast::Type;

use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// An array literal, its elements coerced to the declared element type.
    pub fn array(&mut self, node: &ArrayExpression) -> Value<'context> {
        let result_slang_type = node.get_type().expect("slang types every array literal");
        let element_slang_type = match &result_slang_type {
            Type::FixedSizeArray(fixed_array_type) => fixed_array_type.element_type(),
            Type::Array(array_type) => array_type.element_type(),
            _ => unreachable!(
                "array literal has unexpected result type: {:?}",
                std::mem::discriminant(&result_slang_type)
            ),
        };
        let element_type = self.resolve_type(&element_slang_type, None);
        let element_values = node
            .items()
            .iter()
            .map(|item| self.coerced(&item, element_type))
            .collect::<Vec<_>>();
        Value::array_literal(
            &element_values,
            self.resolve_type(&result_slang_type, None),
            self,
        )
    }
}
