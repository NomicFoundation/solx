//!
//! A base in a linearisation.
//!

///
/// A base in a linearisation, keyed by its standard JSON source path and name.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Base {
    /// The standard JSON source path.
    pub path: String,
    /// The declared name of the definition in `path`.
    pub name: String,
}
