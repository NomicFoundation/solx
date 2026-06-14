//!
//! A bound local variable: its alloca'd pointer and element type.
//!

use melior::ir::Type;
use melior::ir::Value;

/// A bound local variable: the alloca'd pointer holding it and the element type
/// of that pointer (e.g. `ui64` for a `uint64`). Reads produce a `sol.load` of
/// the element type; writes a `sol.store`. Replaces a `(Value, Type)` tuple.
#[derive(Clone, Copy)]
pub struct VariableBinding<'context, 'block> {
    /// The alloca'd pointer holding the variable.
    pub pointer: Value<'context, 'block>,
    /// The element type of the pointer.
    pub element_type: Type<'context>,
}
