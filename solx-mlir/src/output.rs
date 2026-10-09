//!
//! MLIR pipeline output produced by [`crate::Context::finalize_module`].
//!

///
/// The LLVM bitcode codegen compiles for a single contract, and the MLIR text captured for output.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MlirOutput {
    /// Pre-pass Sol dialect text of the deploy module, then the runtime module.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sol_source: Option<String>,
    /// LLVM dialect text of the deploy module.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub deploy_source: String,
    /// LLVM bitcode of the deploy module.
    #[serde(skip)]
    pub deploy_bitcode: Vec<u8>,
    /// Objects the deploy code may embed, its runtime child leading.
    pub deploy_dependencies: solx_utils::Dependencies,
    /// LLVM dialect text of the runtime module.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub runtime_source: String,
    /// LLVM bitcode of the runtime module.
    #[serde(skip)]
    pub runtime_bitcode: Vec<u8>,
    /// Objects the runtime code may embed.
    pub runtime_dependencies: solx_utils::Dependencies,
}

impl MlirOutput {
    /// Joins the outputs of a contract's deploy and runtime modules, whose MLIR text is captured
    /// for both or neither.
    pub fn new(deploy: SegmentOutput, runtime: SegmentOutput) -> Self {
        Self {
            sol_source: deploy
                .sol_source
                .zip(runtime.sol_source)
                .map(|(deploy, runtime)| format!("{deploy}\n{runtime}")),
            deploy_source: deploy.source.unwrap_or_default(),
            deploy_bitcode: deploy.bitcode,
            deploy_dependencies: deploy.dependencies,
            runtime_source: runtime.source.unwrap_or_default(),
            runtime_bitcode: runtime.bitcode,
            runtime_dependencies: runtime.dependencies,
        }
    }
}

///
/// The LLVM bitcode of one code segment of a contract, and the MLIR text captured for output.
///
pub struct SegmentOutput {
    /// Pre-pass Sol dialect text.
    pub sol_source: Option<String>,
    /// LLVM dialect text.
    pub source: Option<String>,
    /// LLVM bitcode.
    pub bitcode: Vec<u8>,
    /// Objects the code may embed.
    pub dependencies: solx_utils::Dependencies,
}
