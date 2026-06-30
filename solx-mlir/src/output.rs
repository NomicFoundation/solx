//!
//! MLIR pipeline output for a single contract.
//!

/// Captured MLIR text for a single contract.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MlirOutput {
    /// Pre-pass Sol dialect text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sol_source: Option<String>,
    /// LLVM dialect text of the deploy module.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub llvm_deploy_source: String,
    /// LLVM dialect text of the runtime module.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub llvm_runtime_source: String,
    /// Cross-contract references: bare object names the linker resolves to deploy bytecode.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
}
