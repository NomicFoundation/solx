//!
//! `solx` Sourcify sweep corpus contract.
//!

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::LazyLock;

use regex::Regex;

/// Settings passed through verbatim: dropping one would turn a real frontend gap into a harness
/// artifact, and the same input must stay valid for the reference compiler.
const PASSTHROUGH_SETTINGS: [&str; 4] = ["evmVersion", "libraries", "remappings", "viaIR"];

/// Version-expression characters only, so `pragma solidity` in a comment never swallows code up
/// to the next `;`.
static VERSION_PRAGMA: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"pragma\s+solidity\s+[0-9xX*.^~<>=|\s-]+;").expect("Always valid")
});

///
/// One corpus record: a verified contract with all its sources.
///
#[derive(Debug, serde::Deserialize)]
pub struct Contract {
    /// The source path whose contracts must get bytecode.
    pub target: String,
    /// The source contents by path.
    pub sources: BTreeMap<String, String>,
    /// The verified compiler settings.
    #[serde(default)]
    pub settings: serde_json::Map<String, serde_json::Value>,
}

impl Contract {
    ///
    /// The standard JSON input: all sources inline under their original paths, the optimizer at
    /// the compiler default, bytecode output only, and every version pragma replaced by a lower
    /// bound of `solidity_version`.
    ///
    pub fn standard_json_input(&self, solidity_version: &str) -> Vec<u8> {
        let pragma = format!("pragma solidity >={solidity_version};");
        let sources: serde_json::Map<String, serde_json::Value> = self
            .sources
            .iter()
            .map(|(source_path, content)| {
                let content = VERSION_PRAGMA.replace_all(content, regex::NoExpand(pragma.as_str()));
                (
                    source_path.to_owned(),
                    serde_json::json!({ "content": content }),
                )
            })
            .collect();

        let mut settings: serde_json::Map<String, serde_json::Value> = self
            .settings
            .iter()
            .filter(|(key, value)| {
                PASSTHROUGH_SETTINGS.contains(&key.as_str())
                    && !value.is_null()
                    && value.as_array().is_none_or(|array| !array.is_empty())
                    && value.as_object().is_none_or(|object| !object.is_empty())
            })
            .map(|(key, value)| (key.to_owned(), value.to_owned()))
            .collect();
        settings.insert(
            "outputSelection".to_owned(),
            serde_json::json!({ "*": { "*": ["evm.bytecode.object", "evm.deployedBytecode.object"] } }),
        );

        let input = serde_json::json!({
            "language": "Solidity",
            "sources": sources,
            "settings": settings,
        });
        serde_json::to_vec(&input).expect("Always valid")
    }
}

impl TryFrom<PathBuf> for Contract {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let string = std::fs::read_to_string(path.as_path())
            .map_err(|error| anyhow::anyhow!("Corpus record {path:?} reading: {error}"))?;
        let contract: Self = serde_json::from_str(string.as_str())
            .map_err(|error| anyhow::anyhow!("Corpus record {path:?} parsing: {error}"))?;
        Ok(contract)
    }
}
