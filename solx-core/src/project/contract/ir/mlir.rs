//!
//! The contract MLIR source code.
//!

///
/// The contract MLIR source code.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MLIR {
    /// LLVM dialect text of this code segment.
    pub source: String,
    /// Cross-contract object dependencies.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
    /// Runtime code object that is only set in deploy code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_code: Option<Box<Self>>,
}
