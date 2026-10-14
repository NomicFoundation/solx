//!
//! `solx` test arguments.
//!

pub mod foundry;
pub mod hardhat;
pub mod solx_tester;
pub mod sourcify;

use clap::Subcommand;

pub use self::foundry::Foundry;
pub use self::hardhat::Hardhat;
pub use self::solx_tester::SolxTester;
pub use self::sourcify::Sourcify;
pub use self::sourcify::SourcifyReport;

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
    /// Compile a Sourcify corpus and record the outcomes.
    Sourcify(Sourcify),
    /// Render the Sourcify sweep report from recorded outcomes.
    SourcifyReport(SourcifyReport),
}
