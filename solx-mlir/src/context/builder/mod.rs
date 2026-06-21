//!
//! The dialect construction handle the `mlir_op!` macros read.
//!
//! [`Builder`] is the `{context, location}` pair every op construction needs —
//! the `mlir_op!` family of macros read these two fields. The dialect emission
//! methods all live on their owning nodes and entities ([`crate::Value`] for Sol,
//! [`crate::YulValue`] for Yul).
//!

pub mod try_fallback_kind;

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
