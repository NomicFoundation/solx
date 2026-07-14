//!
//! Array literal expressions.
//!

use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Value;

use crate::contract::function::expression::Expression;
use crate::r#type::Type;

codegen!(
    /// An array literal, its elements coerced to the declared element type.
    ArrayExpression -> Value |node, scope| {
        let result_slang_type = node.get_type().expect("slang types every array literal");
        let element_slang_type = match &result_slang_type {
            SlangType::FixedSizeArray(fixed_array_type) => fixed_array_type.element_type(),
            SlangType::Array(array_type) => array_type.element_type(),
            _ => unreachable!(
                "array literal has unexpected result type: {:?}",
                std::mem::discriminant(&result_slang_type)
            ),
        };
        let array_type = Type::resolve(&result_slang_type, None, scope);
        let element_type = Type::resolve(&element_slang_type, None, scope);
        let element_values = node
            .items()
            .iter()
            .map(|item| Expression::emit(&item, scope).coerce(element_type, scope))
            .collect::<Vec<_>>();
        Value::array_literal(&element_values, array_type, scope)
    }
);
