//!
//! The Solidity test file.
//!

use std::fs;
use std::io::Read;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

///
/// The Solidity test file.
///
#[derive(Debug, Serialize, Deserialize)]
pub struct TestFile {
    /// The test data used for updating.
    #[serde(skip_serializing)]
    pub data: Option<String>,
    /// The original test file hash. Only for Solidity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// Whether the test is enabled.
    pub enabled: bool,
    /// The test group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// The comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// The optimization modes which all the cases are enabled for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modes: Option<Vec<String>>,
    /// The compiler version the test must be run with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<semver::VersionReq>,
}

impl TryFrom<&Path> for TestFile {
    type Error = anyhow::Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let mut file = fs::File::open(value)?;

        let mut data = String::new();
        file.read_to_string(&mut data)
            .map_err(|error| anyhow::anyhow!("Failed to read test file (2, {file:?}): {error}"))?;

        let hash = Self::md5(data.as_str());

        Ok(Self {
            data: Some(data),
            hash: Some(hash),
            enabled: true,
            group: None,
            comment: None,
            modes: None,
            version: None,
        })
    }
}

impl TestFile {
    ///
    /// Check if the file was changed.
    ///
    pub fn was_changed(&self, path: &Path) -> anyhow::Result<bool> {
        let saved_hash = self
            .hash
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Test file hash is None: {path:?}"))?;
        let mut file = fs::File::open(path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)
            .map_err(|error| anyhow::anyhow!("Failed to read test file (3, {file:?}): {error}"))?;
        let actual_hash = Self::md5(data.as_str());
        Ok(!saved_hash.eq(&actual_hash))
    }

    ///
    /// Returns MD5 hash as hex string.
    ///
    pub fn md5(data: &str) -> String {
        format!("0x{:x}", md5::compute(data.as_bytes()))
    }
}
