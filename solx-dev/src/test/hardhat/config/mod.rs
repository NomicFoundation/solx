//!
//! `solx` Hardhat config.
//!

pub mod comparison;
pub mod compiler;
pub mod project;

use std::collections::BTreeMap;
use std::path::PathBuf;

use self::comparison::Comparison;
use self::compiler::Compiler;
use self::project::Project;

///
/// `solx` Hardhat config.
///
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    /// List of tested rojects.
    pub projects: BTreeMap<String, Project>,
    /// List of downloaded compilers.
    pub compilers: BTreeMap<String, Compiler>,
    /// List of comparisons for Excel diff columns.
    #[serde(default)]
    pub comparisons: Vec<Comparison>,
}

impl TryFrom<PathBuf> for Config {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let string = std::fs::read_to_string(path.as_path()).map_err(|error| {
            anyhow::anyhow!("Hardhat test configuration file {path:?} reading: {error}")
        })?;
        let config: Self = toml::from_str(string.as_str()).map_err(|error| {
            anyhow::anyhow!("Hardhat test configuration file {path:?} parsing: {error}")
        })?;
        Ok(config)
    }
}
