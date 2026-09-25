//!
//! The project-wide data shared by all translation units.
//!

///
/// The project-wide data shared by all translation units.
///
/// Sent to every worker subprocess once, before the per-unit jobs.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Session {
    /// The EVM version to produce bytecode for.
    pub evm_version: Option<solx_utils::EVMVersion>,
    /// Output selection for the compilation.
    pub output_selection: solx_standard_json::InputSelection,
    /// The extra LLVM arguments.
    pub llvm_options: Vec<String>,
    /// The output config for IR artifacts.
    pub output_config: Option<solx_codegen_evm::OutputConfig>,
}

impl Session {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        evm_version: Option<solx_utils::EVMVersion>,
        output_selection: solx_standard_json::InputSelection,
        llvm_options: Vec<String>,
        output_config: Option<solx_codegen_evm::OutputConfig>,
    ) -> Self {
        Self {
            evm_version,
            output_selection,
            llvm_options,
            output_config,
        }
    }
}
