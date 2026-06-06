//!
//! The `MemberCallKind` member-call dispatch enum.
//!

/// The deterministic dispatch class of a member-access call (`x.f(...)`) whose
/// member resolves to a definition (a function or state variable), as opposed to
/// a built-in member (`abi.encode`, `arr.push`, `a.transfer`, `T.wrap`) — those
/// resolve to a [`BuiltIn`](slang_solidity_v2::ast::BuiltIn) and are dispatched
/// by [`CallEmitter::emit_built_in_member_access`](super::CallEmitter::emit_built_in_member_access).
///
/// The SOLE top-level type of this module (D1). Returned by
/// [`CallEmitter::classify_member`](super::CallEmitter::classify_member). #H-M9
/// DROP is baked in: there is NO `LocalMemberCall` — every `x.f()` is external.
pub enum MemberCallKind {
    /// `super.f(...)` — virtual dispatch up the C3 linearisation.
    Super,
    /// A library function call (`L.f(...)`).
    ///
    /// #H8b: `external == compute_selector().is_some()` (an `external`/`public`
    /// library function is delegatecalled; an `internal` one is inlined).
    Library {
        /// Whether the target is an external/public library function.
        external: bool,
    },
    /// A call through a member-resolved function pointer.
    FunctionPointer,
    /// `this.f(...)` — a self external call.
    SelfExternal,
    /// `instance.f(...)` — an external call on another contract instance.
    ExternalInstance,
    /// `this.x` / `this.getter()` — a self getter.
    SelfGetter,
    /// `instance.x` / `instance.getter()` — an external getter.
    ExternalGetter,
}
