//!
//! The contract MLIR source code.
//!

///
/// The contract MLIR source code.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MLIR {
    /// MLIR source code.
    pub source: String,
}
