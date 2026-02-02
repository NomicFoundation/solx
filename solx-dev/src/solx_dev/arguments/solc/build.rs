//!
//! `solx` developer tool solc build arguments.
//!

use clap::Args;

///
/// Solc build arguments.
///
#[derive(Args)]
pub struct Build {
    /// Build type (Release or Debug).
    #[arg(long, default_value = "Release")]
    pub build_type: String,

    /// The solc revision string (e.g., "0.8.33-1.0.2").
    #[arg(long, default_value = "0.8.33-1.0.2")]
    pub revision: String,

    /// Clean the build directory before building.
    #[arg(long)]
    pub clean: bool,

    /// Enable pedantic mode.
    #[arg(long)]
    pub pedantic: bool,

    /// Enable tests.
    #[arg(long)]
    pub tests: bool,

    /// Extra arguments to pass to cmake configure step.
    #[arg(long, num_args = 1..)]
    pub extra_args: Vec<String>,
}
