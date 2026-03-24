//!
//! Function call resolution metadata.
//!

/// Function call resolution metadata for the MLIR builder.
#[derive(Clone)]
pub struct Function {
    /// The mangled MLIR function name.
    pub mlir_name: String,
    /// Number of parameters.
    pub parameter_count: usize,
    /// Number of return values.
    pub return_count: usize,
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
}
