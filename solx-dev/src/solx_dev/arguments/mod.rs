//!
//! `solx` developer tool arguments.
//!

pub mod llvm;
pub mod solc;
pub mod test;

use clap::Parser;

use self::llvm::LLVM;
use self::solc::Solc;
use self::test::Test;

///
/// `solx` developer tool arguments.
///
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub enum Arguments {
    /// Build LLVM with specified options.
    #[command(subcommand)]
    LLVM(LLVM),

    /// Build and manage solc libraries.
    #[command(subcommand)]
    Solc(Solc),

    /// Runs tests and benchmarks on specified projects.
    #[command(subcommand)]
    Test(Test),
}
