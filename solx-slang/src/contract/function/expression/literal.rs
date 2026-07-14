//!
//! Literal expressions: integer numbers, the boolean keywords, and strings.
//!

use solx_mlir::Value;

codegen!(
    DecimalNumberExpression | HexNumberExpression -> Value |node, scope| {
        Value::constant_from_bigint(
            &node
                .integer_value()
                .expect("an integer literal evaluates to an integer"),
            codegen!(@result_type IntegerLiteral, node, scope),
            scope,
        )
    }

    TrueKeyword -> Value |_node, scope| { Value::boolean(true, scope) }

    FalseKeyword -> Value |_node, scope| { Value::boolean(false, scope) }

    StringExpression {
        -> Value |node, scope| {
            Value::string_literal(&Self::text(node), scope)
        }

        /// The literal's UTF-8 text.
        pub fn text(node: &slang_solidity_v2::ast::StringExpression) -> String {
            String::from_utf8(node.value()).expect("slang validates string literals are UTF-8")
        }
    }
);
