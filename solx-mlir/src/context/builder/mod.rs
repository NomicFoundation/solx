//!
//! The Sol-dialect construction handle the `mlir_op!` macros read.
//!
//! [`Builder`] is the `{context, location}` pair every op construction needs —
//! the `mlir_op!` family of macros read these two fields. The Sol-dialect
//! emission methods have all dissolved onto their owning nodes and entities; the
//! Yul cluster ([`yul`]) is the remaining tenant, pending a `YulContext` peer.
//!

pub mod try_fallback_kind;
pub mod yul;

use melior::ir::Location;

/// The `{context, location}` handle the `mlir_op!` macros read.
pub struct Builder<'context> {
    /// The MLIR context with all dialects and translations registered.
    pub context: &'context melior::Context,
    /// Cached unknown source location.
    pub unknown_location: Location<'context>,
}

impl<'context> Builder<'context> {
    /// Creates a new builder with the cached unknown location.
    pub fn new(context: &'context melior::Context) -> Self {
        Self {
            context,
            unknown_location: Location::unknown(context),
        }
    }
}
