//!
//! `solx` Sourcify sweep config.
//!

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::test::hardhat::config::compiler::Compiler;

///
/// `solx` Sourcify sweep config.
///
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    /// The pinned corpus release.
    pub corpus: Corpus,
    /// The candidate and reference compilers.
    pub compilers: BTreeMap<String, Compiler>,
}

///
/// The pinned corpus release.
///
#[derive(Debug, serde::Deserialize)]
pub struct Corpus {
    /// The `.tar.xz` release asset URL.
    pub url: String,
    /// The SHA-256 of the release asset.
    pub sha256: String,
}

impl Config {
    ///
    /// The enabled compiler marked as the correctness candidate.
    ///
    pub fn candidate(&self) -> anyhow::Result<&Compiler> {
        self.compilers
            .values()
            .find(|compiler| !compiler.disabled && compiler.is_correctness_candidate)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No candidate compiler specified in the Sourcify sweep configuration"
                )
            })
    }

    ///
    /// The enabled compiler marked as the correctness reference.
    ///
    pub fn reference(&self) -> anyhow::Result<&Compiler> {
        self.compilers
            .values()
            .find(|compiler| !compiler.disabled && compiler.is_correctness_reference)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No reference compiler specified in the Sourcify sweep configuration"
                )
            })
    }
}

impl TryFrom<PathBuf> for Config {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let string = std::fs::read_to_string(path.as_path()).map_err(|error| {
            anyhow::anyhow!("Sourcify sweep configuration file {path:?} reading: {error}")
        })?;
        let config: Self = toml::from_str(string.as_str()).map_err(|error| {
            anyhow::anyhow!("Sourcify sweep configuration file {path:?} parsing: {error}")
        })?;
        Ok(config)
    }
}
