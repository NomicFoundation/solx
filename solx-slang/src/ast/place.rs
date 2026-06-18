//!
//! A storable location: an address pointer plus its element type.
//!

use melior::ir::Type;
use melior::ir::Value;

/// The lvalue an assignable expression denotes — the `!sol.ptr` address and the
/// MLIR element type loaded from or stored to it, without any `sol.load` /
/// `sol.store`.
pub struct Place<'context, 'block> {
    /// The address the element lives at.
    pub address: Value<'context, 'block>,
    /// The MLIR element type at the address.
    pub element_type: Type<'context>,
}
