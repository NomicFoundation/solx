//!
//! solx tests updater arguments.
//!

use std::path::PathBuf;

use clap::Parser;

///
/// solx tests updater arguments.
///
#[derive(Debug, Parser)]
#[command(about, long_about = None)]
pub struct Arguments {
    /// Source directory of changed tests.
    #[arg(
        short,
        long,
        default_value = "solx-solidity/test/libsolidity/semanticTests"
    )]
    pub source: PathBuf,

    /// Path of the tests' index.
    #[arg(short, long, default_value = "solidity.yaml")]
    pub index: PathBuf,
}
