//!
//! `solx` test arguments.
//!

pub mod foundry;
pub mod hardhat;
pub mod solx_tester;

use clap::Subcommand;

pub use self::foundry::Foundry;
pub use self::hardhat::Hardhat;
pub use self::solx_tester::SolxTester;

///
/// `solx` test arguments.
///
#[derive(Subcommand)]
pub enum Test {
    /// Run Hardhat test projects.
    Hardhat(Hardhat),
    /// Run Foundry test projects.
    Foundry(Foundry),
    /// Run integration tests with solx-tester.
    SolxTester(SolxTester),
}
