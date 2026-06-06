//!
//! Yul EVM-opcode intrinsic lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::BuiltIn;

use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Emits a Yul EVM-opcode intrinsic, dispatching on the TYPED
    /// `BuiltIn::Yul*` variant (R8-9: 83 verified variants via
    /// `resolve_to_built_in()`, NOT `match name: &str` — this is a MECHANICAL
    /// Rule-7 conversion, not a carve-out). A4: unsupported opcodes (e.g.
    /// `verbatim`) are a LOUD residual.
    pub fn emit_yul_intrinsic(
        &self,
        opcode: BuiltIn,
        arguments: &[Value<'context, 'block>],
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let _ = (opcode, arguments, block);
        unimplemented!("yul intrinsic")
    }
}
