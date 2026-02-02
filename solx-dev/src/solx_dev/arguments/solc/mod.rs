//!
//! `solx` developer tool solc arguments.
//!

pub mod build;

use clap::Subcommand;

use self::build::Build;

///
/// Solc subcommand.
///
#[derive(Subcommand)]
pub enum Solc {
    /// Build the solc libraries.
    Build(Build),
}
