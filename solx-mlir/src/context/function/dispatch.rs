//!
//! The internal dispatch attribute a `sol.func` carries.
//!

use crate::FunctionKind;

/// The internal dispatch attribute a `sol.func` carries, if any.
pub enum FunctionDispatch {
    /// The identifier an internal function pointer dispatches to; never zero, which the dialect
    /// reserves for the null function pointer.
    Identifier(u64),
    /// The dialect kind of a constructor, fallback or receive function.
    Kind(FunctionKind),
    /// The function is reached by its symbol alone, as a getter is through its ABI selector and a
    /// base constructor from the constructor before it in the chain.
    Symbol,
}
