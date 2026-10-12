//!
//! The visibility of a function in the debug symbol table.
//!

///
/// The visibility of a function.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// `public`.
    Public,
    /// `external`.
    External,
    /// `internal`.
    Internal,
    /// `private`.
    Private,
}
