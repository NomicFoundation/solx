//!
//! `solx` developer tool solc build arguments.
//!

use clap::Args;

use crate::BuildType;

///
/// Solc build arguments.
///
#[derive(Args)]
pub struct Build {
    /// Build type (`Debug`, `Release`, `RelWithDebInfo`, or `MinSizeRel`).
    #[arg(long, default_value_t = BuildType::Release)]
    pub build_type: BuildType,

    /// Clean the build directory before building.
    #[arg(long)]
    pub clean: bool,

    /// Enable pedantic mode.
    #[arg(long)]
    pub pedantic: bool,

    /// Enable tests.
    #[arg(long)]
    pub tests: bool,

    /// Boost version to use.
    #[arg(long)]
    pub boost_version: Option<String>,

    /// Download and build Boost before building solc.
    #[arg(long)]
    pub build_boost: bool,

    /// Enable MLIR support (requires LLVM built with MLIR).
    #[arg(long)]
    pub enable_mlir: bool,

    /// Use GCC compiler instead of clang.
    #[arg(long)]
    pub use_gcc: bool,

    /// Extra arguments to pass to cmake configure step.
    #[arg(long, num_args = 1..)]
    pub extra_args: Vec<String>,
}
