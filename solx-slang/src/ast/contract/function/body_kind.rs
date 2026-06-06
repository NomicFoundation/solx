//!
//! The `BodyKind` function/modifier-body discriminant enum.
//!

/// Which form of a function [`FunctionEmitter::emit_sol_inner`] lowers.
///
/// The SOLE top-level type of this module (D1: every Rule-12 discriminant in its
/// own module). Replaces no mode-bool; it disambiguates a normal function
/// emission from the unwrapped `$body` of a modified function.
///
/// [`FunctionEmitter::emit_sol_inner`]: super::FunctionEmitter::emit_sol_inner
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BodyKind {
    /// A normal function emission: public selector and modifier wrapping.
    Function,
    /// The unwrapped body of a modified function, emitted as a separate internal
    /// `sol.func` (the `$body` symbol) — no selector, no modifier wrapping, with
    /// the return values threaded in as trailing parameters.
    ModifierBody,
}
