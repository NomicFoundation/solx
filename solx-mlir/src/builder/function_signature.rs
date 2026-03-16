//!
//! Function signature metadata for internal call resolution.
//!

/// Function signature info for internal call resolution.
#[derive(Clone)]
pub(crate) struct FunctionSignature {
    /// The mangled MLIR function name.
    mlir_name: String,
    /// Number of parameters.
    parameter_count: usize,
    /// Whether the function returns a value.
    has_returns: bool,
}

impl FunctionSignature {
    /// Creates a new function signature.
    pub(crate) fn new(mlir_name: String, parameter_count: usize, has_returns: bool) -> Self {
        Self {
            mlir_name,
            parameter_count,
            has_returns,
        }
    }

    /// Returns the mangled MLIR function name.
    pub(crate) fn mlir_name(&self) -> &str {
        &self.mlir_name
    }

    /// Returns the number of parameters.
    pub(crate) fn parameter_count(&self) -> usize {
        self.parameter_count
    }

    /// Returns whether the function returns a value.
    pub(crate) fn has_returns(&self) -> bool {
        self.has_returns
    }
}
