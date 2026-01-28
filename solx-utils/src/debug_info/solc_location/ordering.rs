//!
//! solc AST source code location ordering.
//!

///
/// solc AST source code location ordering.
///
pub enum Ordering {
    /// AST ordering: start offset, length, source ID.
    Ast,
    /// Yul ordering: source ID, start offset, end offset.
    Yul,
}
