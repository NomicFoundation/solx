//!
//! The contract source code.
//!

pub mod llvm_ir;
pub mod mlir;

use self::llvm_ir::LLVMIR;
use self::mlir::MLIR;

///
/// The contract source code.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum IR {
    /// The LLVM IR source code.
    LLVMIR(LLVMIR),
    /// The MLIR source code.
    MLIR(MLIR),
}

impl From<LLVMIR> for IR {
    fn from(inner: LLVMIR) -> Self {
        Self::LLVMIR(inner)
    }
}

impl From<MLIR> for IR {
    fn from(inner: MLIR) -> Self {
        Self::MLIR(inner)
    }
}
