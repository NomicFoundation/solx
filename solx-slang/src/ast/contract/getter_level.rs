//!
//! The `GetterLevel` indexed-getter step enum.
//!

use melior::ir::Type;

/// One nesting level of an indexed (mapping/array) state-variable getter,
/// consumed in order to chain the storage access from the base slot.
///
/// The SOLE top-level type of this module (D1: every Rule-12 dispatch enum sits
/// in its own module). Produced by `GetterAbi::indexed_getter_levels` and walked
/// by `GetterAbi::emit_getter_access_chain`.
pub enum GetterLevel<'context> {
    /// `sol.map` over a key; carries the mapped-slot reference type.
    Mapping(Type<'context>),
    /// Bounds-checked `sol.gep` over an index; carries the element type and,
    /// for fixed arrays, the static size (dynamic arrays: `None`).
    Array(Type<'context>, Option<u64>),
}
