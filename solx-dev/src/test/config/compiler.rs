//!
//! `solx` test compiler.
//!

use std::collections::BTreeMap;

///
/// `solx` test compiler.
///
#[derive(Debug, serde::Deserialize)]
pub struct Compiler {
    /// Compiler description to display.
    #[serde(default)]
    pub description: String,
    /// Compiler name to display.
    pub name: String,
    /// Compiler path.
    pub path: String,
    /// Solidity/solc version, if different from the current commit.
    pub solidity_version: Option<String>,
    /// Whether the compiler is a correctness reference.
    #[serde(default)]
    pub is_correctness_reference: bool,
    /// Whether the compiler is a correctness candidate.
    #[serde(default)]
    pub is_correctness_candidate: bool,
    /// Whether the compiler is disabled.
    #[serde(default)]
    pub disabled: bool,
}

impl Compiler {
    ///
    /// The enabled compiler marked as the correctness candidate.
    ///
    pub fn candidate<'a>(
        compilers: &'a BTreeMap<String, Self>,
        configuration: &str,
    ) -> anyhow::Result<&'a Self> {
        compilers
            .values()
            .find(|compiler| !compiler.disabled && compiler.is_correctness_candidate)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No candidate compiler specified in the {configuration} configuration"
                )
            })
    }

    ///
    /// The enabled compiler marked as the correctness reference.
    ///
    pub fn reference<'a>(
        compilers: &'a BTreeMap<String, Self>,
        configuration: &str,
    ) -> anyhow::Result<&'a Self> {
        compilers
            .values()
            .find(|compiler| !compiler.disabled && compiler.is_correctness_reference)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No reference compiler specified in the {configuration} configuration"
                )
            })
    }
}
