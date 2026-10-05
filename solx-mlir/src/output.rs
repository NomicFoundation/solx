//!
//! MLIR pipeline output produced by [`crate::Context::finalize_module`].
//!

///
/// Captured MLIR text for a single contract.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MlirOutput {
    /// Pre-pass Sol dialect text of the deploy module, then the runtime module.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sol_source: Option<String>,
    /// LLVM dialect text of the deploy module.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub deploy_source: String,
    /// Objects the deploy code may embed, its runtime child leading.
    pub deploy_dependencies: solx_utils::Dependencies,
    /// LLVM dialect text of the runtime module.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub runtime_source: String,
    /// Objects the runtime code may embed.
    pub runtime_dependencies: solx_utils::Dependencies,
}

impl MlirOutput {
    /// Joins the outputs of a contract's deploy and runtime modules, whose Sol text is captured
    /// for both or neither.
    pub fn new(deploy: SegmentOutput, runtime: SegmentOutput) -> Self {
        Self {
            sol_source: deploy
                .sol_source
                .zip(runtime.sol_source)
                .map(|(deploy, runtime)| format!("{deploy}\n{runtime}")),
            deploy_source: deploy.source,
            deploy_dependencies: deploy.dependencies,
            runtime_source: runtime.source,
            runtime_dependencies: runtime.dependencies,
        }
    }
}

///
/// Captured MLIR text for one code segment of a contract.
///
pub struct SegmentOutput {
    /// Pre-pass Sol dialect text.
    pub sol_source: Option<String>,
    /// LLVM dialect text.
    pub source: String,
    /// Objects the code may embed.
    pub dependencies: solx_utils::Dependencies,
}
