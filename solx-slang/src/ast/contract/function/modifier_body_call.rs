//!
//! The `ModifierBodyCall` modifier-stage hand-off record.
//!

use melior::ir::BlockLike;
use melior::ir::Type;
use melior::ir::Value;

use solx_mlir::Builder;

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

impl<'context, 'block> ModifierBodyCall<'context, 'block> {
    /// Emits the hand-off: call the downstream `sol.func` ([`symbol`](Self::symbol))
    /// with the forwarded parameters followed by the *current* return-slot values,
    /// then store the call's results back into the return slots so the modifier
    /// tail and the epilogue observe them. The single source for this sequence,
    /// shared by the `_;` placeholder ([`StatementEmitter`]) and the public
    /// entry's outermost-stage call ([`FunctionEmitter::emit_modified_body`]).
    ///
    /// [`StatementEmitter`]: crate::ast::contract::function::statement::StatementEmitter
    /// [`FunctionEmitter::emit_modified_body`]: crate::ast::contract::function::FunctionEmitter::emit_modified_body
    ///
    /// # Errors
    ///
    /// Returns an error if a return-slot load or the downstream call cannot be
    /// lowered.
    pub fn emit<Block>(&self, builder: &Builder<'context>, block: &Block) -> anyhow::Result<()>
    where
        Block: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let mut operands = self.forward_params.clone();
        for (slot, result_type) in self.return_slots.iter().zip(self.result_types.iter()) {
            if let Some(pointer) = slot {
                operands.push(builder.emit_sol_load(*pointer, *result_type, block)?);
            }
        }
        let results =
            builder.emit_sol_call_results(&self.symbol, &operands, &self.result_types, block)?;
        for (slot, value) in self.return_slots.iter().zip(results) {
            if let Some(pointer) = slot {
                builder.emit_sol_store(value, *pointer, block);
            }
        }
        Ok(())
    }
}
