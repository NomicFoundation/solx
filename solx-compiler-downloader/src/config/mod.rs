//!
//! The compiler downloader config.
//!

pub mod compiler_list;
pub mod executable;

use self::executable::Executable;

///
/// The compiler downloader config.
///
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    /// Compiler executables to download.executables.
    pub executables: Vec<Executable>,
}
