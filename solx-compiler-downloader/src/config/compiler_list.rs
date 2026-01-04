//!
//! The compiler JSON list metadata.
//!

use std::collections::BTreeMap;
use std::str::FromStr;

use colored::Colorize;
use reqwest::Url;

///
/// The compiler JSON list metadata.
///
#[derive(Debug, serde::Deserialize)]
pub struct CompilerList {
    /// The collection of compiler releases.
    pub releases: BTreeMap<String, String>,
}

impl TryFrom<&str> for CompilerList {
    type Error = anyhow::Error;

    fn try_from(url: &str) -> Result<Self, Self::Error> {
        println!(
            " {} compiler bin JSON {url:?}",
            "Downloading".bright_green().bold(),
        );

        let url = Url::from_str(url)
            .map_err(|error| anyhow::anyhow!("URL `{url}` parsing error: {error}"))?;
        let list: Self = reqwest::blocking::get(url.clone())
            .map_err(|error| anyhow::anyhow!("Compiler bin JSON {url:?} download error: {error}"))?
            .json()
            .map_err(|error| anyhow::anyhow!("Compiler bin JSON {url:?} parsing error: {error}"))?;
        Ok(list)
    }
}
