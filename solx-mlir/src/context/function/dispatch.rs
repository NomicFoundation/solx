//!
//! The attribute a `sol.func` is reached through.
//!

use crate::FunctionKind;

/// The attribute a `sol.func` is reached through, one or the other and never both: a regular
/// function is tagged by the identifier an internal pointer dispatches to, while the dialect names
/// every other kind.
#[derive(Clone, Copy)]
pub enum FunctionDispatch {
    /// The identifier an internal function pointer dispatches to.
    Identifier(usize),
    /// The dialect kind of a constructor, fallback or receive function.
    Kind(FunctionKind),
}
