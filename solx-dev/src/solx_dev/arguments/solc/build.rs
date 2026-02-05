//!
//! `solx` developer tool solc build arguments.
//!

use clap::Args;

///
/// Solc build arguments.
///
#[derive(Args)]
pub struct Build {
    /// Build type (`Debug`, `Release`, `RelWithDebInfo`, or `MinSizeRel`).
    #[arg(long, default_value_t = solx_dev::BuildType::Release)]
    pub build_type: solx_dev::BuildType,

    /// Clean the build directory before building.
    #[arg(long)]
    pub clean: bool,

    /// Enable pedantic mode.
    #[arg(long)]
    pub pedantic: bool,

    /// Enable tests.
    #[arg(long)]
    pub tests: bool,

    /// Build local Boost (uses default version unless --boost-version is set).
    #[arg(long)]
    pub build_boost: bool,

    /// Boost version to build when --build-boost is enabled.
    #[arg(long, value_name = "VERSION")]
    pub boost_version: Option<String>,

    /// Enable MLIR support (requires LLVM built with MLIR).
    #[arg(long)]
    pub enable_mlir: bool,

    /// Use GCC compiler instead of clang.
    #[arg(long)]
    pub use_gcc: bool,

    /// Use ccache for faster compilation.
    #[arg(long)]
    pub ccache: bool,

    /// Extra arguments to pass to cmake configure step.
    #[arg(long, num_args = 1..)]
    pub extra_args: Vec<String>,
}
