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
    /// Bare object names of contracts this segment references via cross-
    /// contract ops (`sol.new` and friends). Populated during Slang→MLIR
    /// emission and used by the assembler to pull in dependency bytecodes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
    /// Runtime code object that is only set in deploy code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_code: Option<Box<Self>>,
}
