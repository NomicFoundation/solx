//!
//! The LLVM module build.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;

///
/// The LLVM module build.
///
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Build {
    /// Bytecode.
    pub bytecode: Option<Vec<u8>>,
    /// Text assembly.
    pub assembly: Option<String>,
    /// Unoptimized LLVM IR (solx internal representation).
    pub llvm_ir_unoptimized: Option<String>,
    /// Optimized LLVM IR (solx internal representation).
    pub llvm_ir: Option<String>,
    /// Mapping with immutables.
    pub immutables: Option<BTreeMap<String, BTreeSet<u64>>>,
    /// Whether the size fallback has been activated.
    pub is_size_fallback: bool,
    /// Warnings produced during compilation.
    pub warnings: Vec<solx_utils::Warning>,
}

impl Build {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        bytecode: Option<Vec<u8>>,
        assembly: Option<String>,
        llvm_ir_unoptimized: Option<String>,
        llvm_ir: Option<String>,
        immutables: Option<BTreeMap<String, BTreeSet<u64>>>,
        is_size_fallback: bool,
        warnings: Vec<solx_utils::Warning>,
    ) -> Self {
        Self {
            bytecode,
            assembly,
            llvm_ir_unoptimized,
            llvm_ir,
            immutables,
            is_size_fallback,
            warnings,
        }
    }
}
