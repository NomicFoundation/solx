//!
//! `solx` LLVM builder platforms.
//!

pub mod aarch64_linux_gnu;
pub mod aarch64_macos;
pub mod shared;
pub mod x86_64_linux_gnu;
pub mod x86_64_macos;
pub mod x86_64_windows_gnu;

use std::str::FromStr;

///
/// The list of platforms used as constants.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    /// The EVM back end developed by Matter Labs.
    EVM,
}

impl FromStr for Platform {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "EVM" => Ok(Self::EVM),
            value => Err(format!("Unsupported platform: `{value}`")),
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::EVM => write!(f, "EVM"),
        }
    }
}
