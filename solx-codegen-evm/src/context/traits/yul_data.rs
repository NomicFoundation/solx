//!
//! The LLVM IR Yul data trait.
//!

///
/// The LLVM IR Yul data trait.
///
pub trait IYulData {
    ///
    /// Resolves the full contract path by the Yul object identifier.
    ///
    fn resolve_path(&self, identifier: &str) -> Option<&str>;
}
