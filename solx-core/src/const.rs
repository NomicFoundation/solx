//!
//! Solidity compiler constants.
//!

/// The default executable name.
pub static DEFAULT_EXECUTABLE_NAME: &str = "slang";

/// The default package description.
pub static DEFAULT_PACKAGE_DESCRIPTION: &str = "LLVM-based Solidity compiler for the EVM";

/// The `solc` CBOR metadata tag.
pub static SOLC_METADATA_TAG: &str = "solc";

/// The worker thread stack size.
pub const WORKER_THREAD_STACK_SIZE: usize = 64 * 1024 * 1024;
