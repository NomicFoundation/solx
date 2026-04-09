//!
//! Function call resolution metadata.
//!

use melior::ir::Type;

/// Function call resolution metadata for the MLIR builder.
#[derive(Clone)]
pub struct Function<'context> {
    /// The mangled MLIR function name.
    pub mlir_name: String,
    /// Number of parameters.
    pub parameter_count: usize,
    /// Return types (MLIR-interned, exact types from the function signature).
    pub return_types: Vec<Type<'context>>,
}

impl<'context> Function<'context> {
    /// Creates a new function metadata entry.
    pub fn new(
        mlir_name: String,
        parameter_count: usize,
        return_types: Vec<Type<'context>>,
    ) -> Self {
        Self {
            mlir_name,
            parameter_count,
            return_types,
        }
    }
}
