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
    /// Number of return values.
    return_count: usize,
}

impl FunctionSignature {
    /// Creates a new function signature.
    pub(crate) fn new(mlir_name: String, parameter_count: usize, return_count: usize) -> Self {
        Self {
            mlir_name,
            parameter_count,
            return_count,
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

    /// Returns the number of return values.
    pub(crate) fn return_count(&self) -> usize {
        self.return_count
    }
}
