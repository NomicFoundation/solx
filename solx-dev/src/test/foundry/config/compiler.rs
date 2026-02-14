//!
//! `solx` Foundry compiler.
//!

///
/// `solx` Foundry compiler.
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
