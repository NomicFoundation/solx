//!
//! How far an object is on its way to bytecode.
//!

///
/// How far an object is on its way to bytecode.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum State {
    /// Compiled to an ELF object that still names the objects it embeds.
    Compiled,
    /// Assembled with the objects it embeds; library addresses are still unresolved.
    Assembled,
    /// Linked to bytecode.
    Linked,
}
