//!
//! The `ModifierBodyCall` modifier-stage hand-off record.
//!

use melior::ir::BlockLike;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::FlatSymbolRefAttribute;

use solx_mlir::Builder;
use solx_mlir::ods::sol::CallOperation;

/// The hand-off from a modifier stage to the wrapped function body.
///
/// The SOLE top-level type of this module (§2a), homed here rather than inline in
/// `statement/mod.rs` (which would force a second top-level type there). A
/// modifier stage's `_` placeholder lowers to a call of the internal
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
    /// shared by the `_;` placeholder ([`StatementContext`]) and the public
    /// entry's outermost-stage call ([`FunctionEmitter::emit_modified_body`]).
    ///
    /// [`StatementContext`]: crate::ast::contract::function::statement::StatementContext
    /// [`FunctionEmitter::emit_modified_body`]: crate::ast::contract::function::FunctionEmitter::emit_modified_body
    pub fn emit<Block>(&self, builder: &Builder<'context>, block: &Block)
    where
        Block: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let mut operands = self.forward_params.clone();
        for (slot, result_type) in self.return_slots.iter().zip(self.result_types.iter()) {
            if let Some(pointer) = slot {
                operands.push(
                    crate::ast::Pointer::new(*pointer)
                        .load(crate::ast::Type::new(*result_type), builder, block)
                        .into_mlir(),
                );
            }
        }
        // The synthetic `$body` / next-stage `sol.func` is not a registered
        // function, so its call is built here rather than through `Function::call`.
        let operation = block.append_operation(sol_op_build!(
            builder,
            CallOperation
                .callee(FlatSymbolRefAttribute::new(builder.context, &self.symbol))
                .outs(&self.result_types)
                .operands(&operands)
        ));
        let results = (0..self.result_types.len()).map(|index| {
            operation
                .result(index)
                .expect("sol.call produces its declared result count")
        });
        for (slot, value) in self.return_slots.iter().zip(results) {
            if let Some(pointer) = slot {
                crate::ast::Pointer::new(*pointer).store(
                    crate::ast::Value::new(value.into()),
                    builder,
                    block,
                );
            }
        }
    }
}
