//!
//! `solx` test arguments.
//!

pub mod foundry;

use clap::Subcommand;

use self::foundry::Foundry;

///
/// `solx` test arguments.
///
#[derive(Subcommand)]
pub enum Test {
    /// Run Foundry test projects.
    Foundry(Foundry),
}
