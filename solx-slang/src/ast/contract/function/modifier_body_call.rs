//!
//! The `ModifierBodyCall` modifier-stage hand-off record.
//!

use melior::ir::Type;
use melior::ir::Value;

/// The hand-off from a modifier stage to the wrapped function body.
///
/// The SOLE top-level type of this module (§2a): the oracle declared this struct
/// inline in `statement/mod.rs` (a forced second top-level type); the recut homes
/// it here. A modifier stage's `_` placeholder lowers to a call of the internal
/// `$body` (or next-stage) `sol.func`, forwarding the wrapping function's
/// parameters and storing the call results into its return slots.
pub struct ModifierBodyCall<'context, 'block> {
    /// Symbol of the internal `sol.func` holding the wrapped body / next stage.
    pub symbol: String,
    /// The downstream `sol.func`'s declared result types.
    pub result_types: Vec<Type<'context>>,
    /// The wrapping function's parameters, forwarded to the body call.
    pub forward_params: Vec<Value<'context, 'block>>,
    /// The wrapping function's return slots; the body call's results are stored
    /// here so the modifier tail and epilogue observe them.
    pub return_slots: Vec<Option<Value<'context, 'block>>>,
}
