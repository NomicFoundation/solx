//!
//! EVM codegen constants.
//!

/// The LLVM framework version.
pub const LLVM_VERSION: semver::Version = semver::Version::new(19, 1, 0);

/// The entry function name.
pub const ENTRY_FUNCTION_NAME: &str = "__entry";

/// The deployed object identifier suffix used by both the Yul AST and the
/// Sol-to-LLVM pass output.
pub static DEPLOYED_OBJECT_SUFFIX: &str = "_deployed";

/// Library deploy address Yul identifier.
pub static LIBRARY_DEPLOY_ADDRESS_TAG: &str = "library_deploy_address";

/// The deploy bytecode size limit.
pub const DEPLOY_CODE_SIZE_LIMIT: usize = 49152;

/// The runtime bytecode size limit.
pub const RUNTIME_CODE_SIZE_LIMIT: usize = 24576;

/// The `solc` user memory offset.
pub const SOLC_USER_MEMORY_OFFSET: u64 = 128;
