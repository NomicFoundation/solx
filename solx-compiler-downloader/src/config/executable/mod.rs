//!
//! The compiler downloader executable config.
//!

pub mod protocol;

use std::collections::HashMap;

use self::protocol::Protocol;

///
/// The compiler downloader executable config.
///
#[derive(Debug, serde::Deserialize)]
pub struct Executable {
    /// Whether downloading the executable is enabled.
    pub is_enabled: bool,
    /// The downloading protocol.
    pub protocol: Protocol,
    /// The downloaded data source.
    pub source: String,
    /// The downloaded executable file destination.
    pub destination: String,
    /// Version key, if applicable.
    pub version: Option<String>,
    /// Compiler platform directory names.
    pub platforms: Option<HashMap<String, String>>,
}

impl Executable {
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
