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
    /// The input contract language.
    pub language: solx_standard_json::InputLanguage,
    /// The `solc` compiler version, used only for Solidity and Yul projects.
    pub solc_version: Option<solx_standard_json::Version>,
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
        language: solx_standard_json::InputLanguage,
        solc_version: Option<solx_standard_json::Version>,
        evm_version: Option<solx_utils::EVMVersion>,
        output_selection: solx_standard_json::InputSelection,
        llvm_options: Vec<String>,
        output_config: Option<solx_codegen_evm::OutputConfig>,
    ) -> Self {
        Self {
            language,
            solc_version,
            evm_version,
            output_selection,
            llvm_options,
            output_config,
        }
    }

    ///
    /// Whether a worker may be reused across this session's units.
    ///
    /// Extra `llvm_options` accumulate in a worker's process-global LLVM cl-option state on
    /// every parse, so a session that carries them runs each unit on a fresh worker.
    ///
    pub fn allows_worker_reuse(&self) -> bool {
        self.llvm_options.is_empty()
    }
}
