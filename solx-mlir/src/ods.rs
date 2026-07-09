//!
//! ODS-generated typed operation wrappers for Sol and Yul dialects.
//!
//! Generated at compile time from the TableGen `.td` files in `solx-llvm`
//! using the [`melior::dialect!`] proc-macro. Provides type-safe operation
//! structs, builders with type-state enforcement, and accessor methods.
//!

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
