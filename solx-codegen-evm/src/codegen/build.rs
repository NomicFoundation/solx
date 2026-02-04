//!
//! The LLVM module build.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::codegen::warning::Warning;

///
/// The LLVM module build.
///
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Build {
    /// Bytecode.
    pub bytecode: Option<Vec<u8>>,
    /// Debug info.
    pub debug_info: Option<Vec<u8>>,
    /// Text assembly.
    pub assembly: Option<String>,
    /// EVM legacy assembly IR (solx internal representation).
    pub evmla: Option<String>,
    /// Ethereal IR (solx internal representation).
    pub ethir: Option<String>,
    /// Unoptimized LLVM IR (solx internal representation).
    pub llvm_ir_unoptimized: Option<String>,
    /// Optimized LLVM IR (solx internal representation).
    pub llvm_ir_optimized: Option<String>,
    /// Mapping with immutables.
    pub immutables: Option<BTreeMap<String, BTreeSet<u64>>>,
    /// Whether the size fallback has been activated.
    pub is_size_fallback: bool,
    /// Warnings produced during compilation.
    pub warnings: Vec<Warning>,
}

impl Build {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        bytecode: Option<Vec<u8>>,
        debug_info: Option<Vec<u8>>,
        assembly: Option<String>,
        evmla: Option<String>,
        ethir: Option<String>,
        llvm_ir_unoptimized: Option<String>,
        llvm_ir_optimized: Option<String>,
        immutables: Option<BTreeMap<String, BTreeSet<u64>>>,
        is_size_fallback: bool,
        warnings: Vec<Warning>,
    ) -> Self {
        Self {
            bytecode,
            debug_info,
            assembly,
            evmla,
            ethir,
            llvm_ir_unoptimized,
            llvm_ir_optimized,
            immutables,
            is_size_fallback,
            warnings,
        }
    }
}
