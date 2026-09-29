//!
//! Whether a function is declared in the source or synthesized by the frontend.
//!

/// Whether a function is declared in the source or synthesized by the frontend, which decides
/// whether its subprogram is artificial.
///
/// A `public` state variable's getter counts as declared: `public` is its declaration, and the
/// getter is in the ABI and callable by name.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FunctionOrigin {
    /// Written in the source.
    Declared,
    /// Added by the frontend with nothing in the source naming it: the constructor of a contract
    /// that declares none. Its subprogram is artificial, at the contract definition.
    Synthesized,
}
