//!
//! EVM codegen constants.
//!

/// The LLVM framework version.
pub const LLVM_VERSION: semver::Version = semver::Version::new(19, 1, 0);

/// The entry function name.
pub const ENTRY_FUNCTION_NAME: &str = "__entry";

/// Library deploy address Yul identifier.
pub static LIBRARY_DEPLOY_ADDRESS_TAG: &str = "library_deploy_address";

/// The `solc` user memory offset.
pub const SOLC_USER_MEMORY_OFFSET: u64 = 128;
