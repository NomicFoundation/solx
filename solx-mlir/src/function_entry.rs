//!
//! Registered contract function for entry-point dispatch.
//!

/// A registered contract function for entry-point dispatch.
#[derive(Clone)]
pub struct FunctionEntry {
    /// The mangled MLIR function name (e.g. `solx.fn.get`).
    pub mlir_name: String,
    /// The 4-byte selector.
    pub selector: [u8; 4],
    /// Number of function parameters (for calldata decoding).
    pub param_count: usize,
    /// Whether the function returns a value.
    pub has_returns: bool,
}

impl FunctionEntry {
    /// Creates a new function entry with explicit parameters.
    pub fn new(
        mlir_name: String,
        selector: [u8; 4],
        param_count: usize,
        has_returns: bool,
    ) -> Self {
        Self {
            mlir_name,
            selector,
            param_count,
            has_returns,
        }
    }

    /// Creates a function entry for a state variable getter (no params, returns i256).
    pub fn getter(mlir_name: String, selector: [u8; 4]) -> Self {
        Self {
            mlir_name,
            selector,
            param_count: 0,
            has_returns: true,
        }
    }
}
