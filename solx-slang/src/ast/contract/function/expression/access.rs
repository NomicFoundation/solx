//!
//! Index access expression lowering: `a[i]`, `m[k]`.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::IndexAccessExpression;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers an index access `base[index]` to a loaded value.
    pub fn emit_index_access(
        &self,
        _index_access: &IndexAccessExpression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        unimplemented!("index access")
    }
}
