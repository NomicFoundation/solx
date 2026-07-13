//!
//! The per-unit job data.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::project::contract::ir::IR as ContractIR;

///
/// The per-unit job data.
///
/// Sent to a worker subprocess for every translation unit, complementing the session data.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Job {
    /// The input contract name.
    pub contract_name: solx_utils::ContractName,
    /// The input contract IR.
    pub contract_ir: ContractIR,
    /// The code segment.
    pub code_segment: solx_utils::CodeSegment,
    /// Solidity debug info.
    pub debug_info: Option<solx_utils::DebugInfo>,
    /// Immutables produced by the runtime code run.
    pub immutables: Option<BTreeMap<String, BTreeSet<u64>>>,
    /// The metadata bytes.
    pub metadata_bytes: Option<Vec<u8>>,
    /// The optimizer settings.
    pub optimizer_settings: solx_codegen_evm::OptimizerSettings,
}

impl Job {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        contract_name: solx_utils::ContractName,
        contract_ir: ContractIR,
        code_segment: solx_utils::CodeSegment,
        debug_info: Option<solx_utils::DebugInfo>,
        immutables: Option<BTreeMap<String, BTreeSet<u64>>>,
        metadata_bytes: Option<Vec<u8>>,
        optimizer_settings: solx_codegen_evm::OptimizerSettings,
    ) -> Self {
        Self {
            contract_name,
            contract_ir,
            code_segment,
            debug_info,
            immutables,
            metadata_bytes,
            optimizer_settings,
        }
    }
}
