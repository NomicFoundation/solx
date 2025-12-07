//!
//! Solidity compiler constants.
//!

/// The default executable name.
pub static DEFAULT_EXECUTABLE_NAME: &str = "solx";

/// The default package description.
pub static DEFAULT_PACKAGE_DESCRIPTION: &str = "LLVM-based Solidity compiler for the EVM";

/// The `solc` CBOR metadata tag.
pub static SOLC_METADATA_TAG: &str = "solc";

/// The `solc` LLVM revision CBOR metadata tag.
pub static SOLC_LLVM_REVISION_METADATA_TAG: &str = "llvm";

/// The worker thread stack size.
pub const WORKER_THREAD_STACK_SIZE: usize = 64 * 1024 * 1024;

/// The default serializing/deserializing buffer size.
pub const DEFAULT_SERDE_BUFFER_SIZE: usize = solx_evm_assembly::Assembly::DEFAULT_SERDE_BUFFER_SIZE;

/// `solx` optimization parameter environment variable name.
pub static SOLX_OPTIMIZATION_ENV: &str = "SOLX_OPTIMIZATION";

/// `solx` optimizater size fallback flag environment variable name.
pub static SOLX_OPTIMIZATION_SIZE_FALLBACK_ENV: &str = "SOLX_OPTIMIZATION_SIZE_FALLBACK";

///
/// The compiler version default function.
///
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}
