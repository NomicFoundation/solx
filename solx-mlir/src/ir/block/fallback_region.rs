//!
//! Fallback region shape for `sol::TryOp`.
//!

use solx_utils::DataLocation;

use crate::Context;
use crate::Type;

/// The shape a declared `sol.try` fallback region takes, which the lowering reads to decide what an
/// unmatched revert does. An undeclared one leaves the region blockless and forwards the revert.
pub enum FallbackRegion {
    /// `catch { .. }`, which swallows the revert without reading its return data.
    Unbound,
    /// `catch (bytes memory data) { .. }`, whose entry block takes the payload as its argument.
    ReturnData,
}

impl FallbackRegion {
    /// The value the clause binds in its region's entry block, absent when it declares no parameter.
    pub fn binding<'context>(self, context: &Context<'context>) -> Option<Type<'context>> {
        match self {
            Self::Unbound => None,
            Self::ReturnData => Some(Type::string(context.melior, DataLocation::Memory)),
        }
    }
}
