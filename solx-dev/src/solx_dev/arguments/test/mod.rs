//!
//! `solx` test arguments.
//!

pub mod foundry;
pub mod hardhat;

use clap::Subcommand;

use self::foundry::Foundry;
use self::hardhat::Hardhat;

///
/// `solx` test arguments.
///
#[derive(Subcommand)]
pub enum Test {
    /// Run Hardhat test projects.
    Hardhat(Hardhat),
    /// Run Foundry test projects.
    Foundry(Foundry),
}
