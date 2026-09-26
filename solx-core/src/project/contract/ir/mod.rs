//!
//! The contract source code.
//!

pub mod evmla;
pub mod llvm_ir;
pub mod yul;

use self::evmla::EVMLegacyAssembly;
use self::llvm_ir::LLVMIR;
use self::yul::Yul;

///
/// The contract source code.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum IR {
    /// The Yul source code.
    Yul(Yul),
    /// The EVM legacy assembly source code.
    EVMLegacyAssembly(EVMLegacyAssembly),
    /// The LLVM IR source code.
    LLVMIR(LLVMIR),
}

impl From<Yul> for IR {
    fn from(inner: Yul) -> Self {
        Self::Yul(inner)
    }
}

impl From<EVMLegacyAssembly> for IR {
    fn from(inner: EVMLegacyAssembly) -> Self {
        Self::EVMLegacyAssembly(inner)
    }
}

impl From<LLVMIR> for IR {
    fn from(inner: LLVMIR) -> Self {
        Self::LLVMIR(inner)
    }
}
