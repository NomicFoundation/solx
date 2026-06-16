//!
//! The `GetterLevel` indexed-getter step enum.
//!

use melior::ir::Type;

/// One nesting level of an indexed (mapping/array) state-variable getter,
/// consumed in order to chain the storage access from the base slot.
pub enum GetterLevel<'context> {
    /// `sol.map` over a key; carries the mapped-slot reference type.
    Mapping(Type<'context>),
    /// Bounds-checked `sol.gep` over an index; carries the element type and,
    /// for fixed arrays, the static size (dynamic arrays: `None`).
    Array(Type<'context>, Option<u64>),
}
