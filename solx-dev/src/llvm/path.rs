//!
//! `solx` LLVM builder constants.
//!

use std::path::PathBuf;

///
/// The LLVM path resolver.
///
pub struct Path {}

impl Path {
    /// The LLVM source directory.
    pub const DIRECTORY_LLVM_SOURCE: &'static str = "./solx-llvm/";

    /// The LLVM target directory.
    pub const DIRECTORY_LLVM_TARGET: &'static str = "./target-llvm/";

    ///
    /// Returns the path to the `llvm` LLVM source module directory.
    ///
    pub fn llvm_module_llvm() -> anyhow::Result<PathBuf> {
        let mut path = PathBuf::from(Self::DIRECTORY_LLVM_SOURCE);
        path.push("llvm");
        crate::utils::absolute_path(path)
    }

    ///
    /// Returns the path to the LLVM final build directory.
    ///
    pub fn llvm_build_final() -> anyhow::Result<PathBuf> {
        let mut path = PathBuf::from(Self::DIRECTORY_LLVM_TARGET);
        path.push("build-final");
        crate::utils::absolute_path(path)
    }

    ///
    /// Returns the path to the LLVM final target directory.
    ///
    pub fn llvm_target_final() -> anyhow::Result<PathBuf> {
        let mut path = PathBuf::from(Self::DIRECTORY_LLVM_TARGET);
        path.push("target-final");
        crate::utils::absolute_path(path)
    }
}
