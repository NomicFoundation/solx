//!
//! The internal dispatch attribute a `sol.func` carries.
//!

use std::num::NonZeroU64;

use crate::FunctionKind;

/// The internal dispatch attribute a `sol.func` carries, if any.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FunctionDispatch {
    /// The identifier an internal function pointer dispatches to; the dialect reserves zero for the
    /// null function pointer.
    Identifier(NonZeroU64),
    /// The dialect kind of a constructor, fallback or receive function.
    Kind(FunctionKind),
    /// A state-variable getter or a base constructor, reached by its symbol alone.
    Symbol,
    /// A modifier, defined as `sol.modifier` and reached by the invocations naming it.
    Modifier,
}
