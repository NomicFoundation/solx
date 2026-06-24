//!
//! The `BodyKind` function/modifier-body discriminant enum.
//!

/// Which form of a function `EmitFunction::emit_inner` emits — disambiguating a
/// normal function emission from the unwrapped `$body` of a modified function.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BodyKind {
    /// A normal function emission: public selector and modifier wrapping.
    Function,
    /// The unwrapped body of a modified function (the `$body` `sol.func`): no selector, no wrapping,
    /// with return values threaded in as trailing parameters.
    ModifierBody,
}
