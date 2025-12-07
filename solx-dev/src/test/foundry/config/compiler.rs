//!
//! `solx` Foundry compiler.
//!

///
/// `solx` Foundry compiler.
///
#[derive(Debug, serde::Deserialize)]
pub struct Compiler {
    /// Compiler path.
    pub path: String,
    /// Compiler description.
    pub description: String,
}
