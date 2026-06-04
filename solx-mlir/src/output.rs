//!
//! MLIR pipeline output produced by [`crate::Context::finalize_module`].
//!

///
/// Captured MLIR text for a single contract.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MlirOutput {
    /// Pre-pass Sol dialect text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sol_source: Option<String>,
    /// LLVM dialect text of the deploy module.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub deploy_source: String,
    /// LLVM dialect text of the runtime module.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub runtime_source: String,
    /// Cross-contract references collected during Slang→MLIR emission.
    ///
    /// Each entry is the bare object name (e.g. `"B"`) of a contract this
    /// module references via `sol.new` or other cross-contract ops. The
    /// linker resolves these to the dependency's deploy bytecode when
    /// assembling.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
}
