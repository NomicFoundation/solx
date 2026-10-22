//!
//! `solx` Sourcify sweep config.
//!

pub mod corpus;

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::test::config::compiler::Compiler;

use self::corpus::Corpus;

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
