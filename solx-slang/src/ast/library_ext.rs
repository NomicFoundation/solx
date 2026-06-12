//!
//! Pure transformations on Slang's [`LibraryDefinition`] AST node.
//!

use slang_solidity_v2::ast::LibraryDefinition;

/// Extension methods on Slang's [`LibraryDefinition`] AST node.
///
/// An extension trait (NOT a slang API); a `pub trait` per the visibility rule
/// (no `pub(crate)`).
pub trait LibraryExt {
    /// The library's linker symbol, the fully-qualified `"<file>:<Library>"`
    /// name solc records in `link_references` — so a linked deploy address and
    /// the `lib_addr` placeholder round-trip. The single source of this format.
    fn link_symbol(&self) -> String;
}

impl LibraryExt for LibraryDefinition {
    fn link_symbol(&self) -> String {
        format!("{}:{}", self.get_file_id(), self.name().name())
    }
}
