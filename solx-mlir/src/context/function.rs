//!
//! Function call resolution metadata.
//!

/// Function call resolution metadata for the MLIR builder.
#[derive(Clone)]
pub struct Function {
    /// The mangled MLIR function name.
    mlir_name: String,
    /// Number of parameters.
    parameter_count: usize,
    /// Number of return values.
    return_count: usize,
}

impl Function {
    /// Creates a new function metadata entry.
    pub fn new(mlir_name: String, parameter_count: usize, return_count: usize) -> Self {
        Self {
            mlir_name,
            parameter_count,
            return_count,
        }
    }

    /// Returns the mangled MLIR function name.
    pub fn mlir_name(&self) -> &str {
        &self.mlir_name
    }

    /// Returns the number of parameters.
    pub fn parameter_count(&self) -> usize {
        self.parameter_count
    }

    /// Returns the number of return values.
    pub fn return_count(&self) -> usize {
        self.return_count
    }
}
