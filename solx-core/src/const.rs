//!
//! Solidity compiler constants.
//!

/// The default executable name.
pub static DEFAULT_EXECUTABLE_NAME: &str = "solx";

/// The default package description.
pub static DEFAULT_PACKAGE_DESCRIPTION: &str = "LLVM-based Solidity compiler for the EVM";

/// The `solc` CBOR metadata tag.
pub static SOLC_METADATA_TAG: &str = "solc";

/// The key of the compiler's own section in the output metadata.
#[cfg(feature = "solc")]
pub static METADATA_SECTION_KEY: &str = "solx";

/// The key of the compiler's own section in the output metadata.
#[cfg(not(feature = "solc"))]
pub static METADATA_SECTION_KEY: &str = "slang";

/// The worker thread stack size.
pub const WORKER_THREAD_STACK_SIZE: usize = 64 * 1024 * 1024;

/// The default serializing/deserializing buffer size.
pub const DEFAULT_SERDE_BUFFER_SIZE: usize = solx_evm_assembly::Assembly::DEFAULT_SERDE_BUFFER_SIZE;
