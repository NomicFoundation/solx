//!
//! The Sol-dialect construction handle the `sol_op!` macros read.
//!
//! [`Builder`] carries the `{context, location}` every op construction needs.
//! Its one remaining emission method, [`Builder::emit_sol_require`], is the
//! conditional revert, pending the conditional-flag macro extension that will
//! let it inline at its producing sites.
//!

pub mod try_fallback_kind;
pub mod yul;

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::Location;
use melior::ir::Value;
use melior::ir::attribute::StringAttribute;

use crate::ods::sol::RequireOperation;

/// The `{context, location}` handle the `sol_op!` macros read.
pub struct Builder<'context> {
    /// The MLIR context with all dialects and translations registered.
    pub context: &'context melior::Context,
    /// Cached unknown source location.
    pub unknown_location: Location<'context>,
}

impl<'context> Builder<'context> {
    /// Creates a new builder with pre-cached types.
    pub fn new(context: &'context melior::Context) -> Self {
        Self {
            context,
            unknown_location: Location::unknown(context),
        }
    }

    /// Emits a `sol.require` conditional revert with an optional message.
    ///
    /// Reverts if `condition` is false. When `msg` is `Some`, the revert
    /// includes the string as a revert reason. Not a terminator — execution
    /// continues after this op when the condition is true.
    pub fn emit_sol_require<'block, B>(
        &self,
        condition: Value<'context, 'block>,
        msg: Option<&str>,
        args: &[Value<'context, 'block>],
        is_call: bool,
        block: &B,
    ) where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let mut builder = RequireOperation::builder(self.context, self.unknown_location)
            .cond(condition)
            .args(args);
        if let Some(msg) = msg {
            builder = builder.msg(StringAttribute::new(self.context, msg));
        }
        if is_call {
            builder = builder.call(Attribute::unit(self.context));
        }
        block.append_operation(builder.build().into());
    }
}
