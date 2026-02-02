//!
//! `solx` developer tool library.
//!

#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::too_many_arguments)]

pub(crate) mod llvm;
pub(crate) mod solc;
pub(crate) mod test;
pub(crate) mod utils;

pub use self::llvm::build as llvm_build;
pub use self::llvm::build_type::BuildType as LLVMBuildType;
pub use self::llvm::ccache_variant::CcacheVariant as LLVMCcacheVariant;
pub use self::llvm::project::Project as LLVMProject;
pub use self::llvm::sanitizer::Sanitizer as LLVMSanitizer;
pub use self::solc::build as solc_build;
pub use self::test::foundry::config::Config as FoundryTestConfig;
pub use self::test::foundry::test as test_foundry;
pub use self::test::hardhat::config::Config as HardhatTestConfig;
pub use self::test::hardhat::test as test_hardhat;
pub use self::utils::*;
