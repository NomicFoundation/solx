//!
//! User-defined value type member built-ins: `T.wrap(x)` and `T.unwrap(x)`.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Lowers `T.wrap(x)` / `T.unwrap(x)`. A user-defined value type is
    /// represented as its underlying type, so both directions are bit-level
    /// identities: emit the single argument coerced to the call's result type
    /// (the wrapped type for `wrap`, the underlying type for `unwrap`).
    pub fn emit_wrap_unwrap(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let argument = arguments
            .iter()
            .next()
            .expect("wrap/unwrap takes a single argument");
        let (value, block) = self.expression_emitter.emit_value(&argument, block)?;
        let target_slang_type = call
            .get_type()
            .expect("the binder types every wrap/unwrap call");
        let builder = &self.expression_emitter.state.builder;
        let target_type = TypeConversion::resolve_slang_type(&target_slang_type, None, builder);
        let value =
            TypeConversion::from_target_type(target_type, builder).emit(value, builder, &block);
        Ok((Some(value), block))
    }
}
