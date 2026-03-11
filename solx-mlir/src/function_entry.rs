//!
//! Registered contract function for entry-point dispatch.
//!

/// A registered contract function for entry-point dispatch.
#[derive(Clone, Debug)]
pub struct FunctionEntry {
    /// The mangled MLIR function name (e.g. `solx.fn.get`).
    mlir_name: String,
    /// The 4-byte selector.
    selector: [u8; 4],
    /// Number of function parameters (for calldata decoding).
    parameter_count: usize,
    /// Whether the function returns a value.
    has_returns: bool,
}

impl FunctionEntry {
    /// Creates a new function entry with explicit parameters.
    pub fn new(
        mlir_name: String,
        selector: [u8; 4],
        parameter_count: usize,
        has_returns: bool,
    ) -> Self {
        Self {
            mlir_name,
            selector,
            parameter_count,
            has_returns,
        }
    }

    /// Creates a function entry for a state variable getter (no params, returns i256).
    pub fn getter(mlir_name: String, selector: [u8; 4]) -> Self {
        Self {
            mlir_name,
            selector,
            parameter_count: 0,
            has_returns: true,
        }
    }

    /// Returns the mangled MLIR function name.
    pub fn mlir_name(&self) -> &str {
        &self.mlir_name
    }

    /// Returns the 4-byte selector.
    pub fn selector(&self) -> [u8; 4] {
        self.selector
    }

    /// Returns the number of function parameters.
    pub fn parameter_count(&self) -> usize {
        self.parameter_count
    }

    /// Returns whether the function returns a value.
    pub fn has_returns(&self) -> bool {
        self.has_returns
    }
}
