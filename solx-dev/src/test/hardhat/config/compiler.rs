//!
//! `solx` Hardhat compiler.
//!

///
/// `solx` Hardhat compiler.
///
#[derive(Debug, serde::Deserialize)]
pub struct Compiler {
    /// Compiler name to display.
    pub name: String,
    /// Compiler path.
    pub path: String,
    /// Compiler description.
    pub description: String,
    /// Whether the compiler is a correctness reference.
    #[serde(default)]
    pub is_correctness_reference: bool,
    /// Whether the compiler is a correctness candidate.
    #[serde(default)]
    pub is_correctness_candidate: bool,
}
