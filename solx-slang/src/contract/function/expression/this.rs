//!
//! The `this` keyword: the enclosing contract as a value.
//!

use solx_mlir::Value;

codegen!(
    ThisKeyword -> Value |_node, scope| {
        Value::this(
            scope
                .current_contract_type
                .expect("sol.this emitted outside a contract"),
            scope,
        )
    }
);
