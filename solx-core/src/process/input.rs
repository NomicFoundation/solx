//!
//! Process for compiling a single compilation unit.
//!
//! The EVM input data.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::project::contract::ir::IR as ContractIR;

///
/// The EVM input data.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Input {
    /// The `solc` compiler version, used only for Solidity and Yul projects.
    pub solc_version: Option<solx_standard_json::Version>,
    /// The input contract name.
    pub contract_name: solx_utils::ContractName,
    /// The input contract IR.
    pub contract_ir: ContractIR,
    /// The code segment.
    pub code_segment: solx_utils::CodeSegment,
    /// The EVM version to produce bytecode for.
    pub evm_version: Option<solx_utils::EVMVersion>,
    /// The mapping of auxiliary identifiers, e.g. Yul object names, to full contract paths.
    pub identifier_paths: BTreeMap<String, String>,
    /// Output selection for the compilation.
    pub output_selection: solx_standard_json::InputSelection,
    /// Immutables produced by the runtime code run.
    pub immutables: Option<BTreeMap<String, BTreeSet<u64>>>,
    /// The metadata bytes.
    pub metadata_bytes: Option<Vec<u8>>,
    /// The optimizer settings.
    pub optimizer_settings: solx_codegen_evm::OptimizerSettings,
    /// The extra LLVM arguments.
    pub llvm_options: Vec<String>,
    /// The debug output config.
    pub debug_config: Option<solx_codegen_evm::DebugConfig>,
}

impl Input {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        solc_version: Option<solx_standard_json::Version>,
        contract_name: solx_utils::ContractName,
        contract_ir: ContractIR,
        code_segment: solx_utils::CodeSegment,
        evm_version: Option<solx_utils::EVMVersion>,
        identifier_paths: BTreeMap<String, String>,
        output_selection: solx_standard_json::InputSelection,
        immutables: Option<BTreeMap<String, BTreeSet<u64>>>,
        metadata_bytes: Option<Vec<u8>>,
        optimizer_settings: solx_codegen_evm::OptimizerSettings,
        llvm_options: Vec<String>,
        debug_config: Option<solx_codegen_evm::DebugConfig>,
    ) -> Self {
        Self {
            solc_version,
            contract_name,
            contract_ir,
            code_segment,
            evm_version,
            identifier_paths,
            output_selection,
            immutables,
            metadata_bytes,
            optimizer_settings,
            llvm_options,
            debug_config,
        }
    }
}
