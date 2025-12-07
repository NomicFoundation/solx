//!
//! The compiler downloader config.
//!

pub mod compiler_list;
pub mod executable;

use std::collections::BTreeMap;
use std::collections::HashMap;

use self::executable::Executable;

///
/// The compiler downloader config.
///
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    /// Compiler executables to download.executables
    pub executables: BTreeMap<String, Executable>,
    /// Compiler platform directory names.
    pub platforms: Option<HashMap<String, String>>,
}

impl Config {
    ///
    /// Returns the remote platform directory name for the specified platform.
    ///
    pub fn get_remote_platform_directory(&self) -> anyhow::Result<String> {
        let platforms = match self.platforms {
            Some(ref platform) => platform,
            None => anyhow::bail!("Platforms are not defined"),
        };

        let platform = if cfg!(target_arch = "x86_64") {
            if cfg!(target_os = "linux") {
                "linux-amd64"
            } else if cfg!(target_os = "macos") {
                "macos-amd64"
            } else if cfg!(target_os = "windows") {
                "windows-amd64"
            } else {
                anyhow::bail!("This platform is not supported!");
            }
        } else if cfg!(target_arch = "aarch64") {
            if cfg!(target_os = "linux") {
                "linux-arm64"
            } else if cfg!(target_os = "macos") {
                "macos-arm64"
            } else {
                anyhow::bail!("This platform is not supported!");
            }
        } else {
            anyhow::bail!("This platform is not supported!");
        };

        platforms
            .get(platform)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Directory for platform `{platform}` is not defined"))
    }
}
