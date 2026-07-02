//!
//! ODS-generated typed operation wrappers for the Sol and Yul dialects, generated at compile time
//! from the `solx-llvm` TableGen `.td` files via the `melior::dialect!` proc-macro.
//!

#![expect(
    missing_docs,
    reason = "melior::dialect! macro generates undocumented items"
)]

melior::dialect! {
    name: "sol",
    files: ["SolOps.td"],
    include_directories: ["mlir/Dialect/Sol"],
}

melior::dialect! {
    name: "yul",
    files: ["YulOps.td"],
    include_directories: ["mlir/Dialect/Yul"],
}
