//!
//! `solx` LLVM tool subcommand.
//!

pub mod build;

use clap::Subcommand;

use self::build::Build;

///
/// `solx` LLVM tool subcommand.
///
#[derive(Subcommand)]
pub enum LLVM {
    /// Build LLVM with specified options.
    Build(Build),
}
