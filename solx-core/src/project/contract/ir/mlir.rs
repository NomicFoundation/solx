//!
//! The contract LLVM bitcode from the MLIR pipeline.
//!

///
/// The contract LLVM bitcode from the MLIR pipeline.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MLIR {
    /// LLVM bitcode of this code segment.
    #[serde(with = "serde_bytes")]
    pub bitcode: Vec<u8>,
    /// Dependencies of this code segment.
    pub dependencies: solx_utils::Dependencies,
    /// Runtime code object that is only set in deploy code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_code: Option<Box<Self>>,
}
