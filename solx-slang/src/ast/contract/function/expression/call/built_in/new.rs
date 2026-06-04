//!
//! Dynamic memory allocation via `new`: `new T[](n)` / `new bytes(n)` /
//! `new string(n)` allocate a fresh, zero-initialised dynamic memory aggregate
//! of `n` elements (or bytes) through `sol.malloc`.
//!
//! Contract creation (`new C(args)`) is a different `new` — it needs the
//! callee's deploy bytecode and is left to the multi-contract domain.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::Type;
use slang_solidity_v2::ast::TypeName;
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Tries to emit a dynamic-allocation `new` — `new T[](n)` / `new bytes(n)`
    /// / `new string(n)` — as a zeroed `sol.malloc` of `n` elements, returning
    /// the memory aggregate. Returns `Ok(None)` when the callee is not a `new`
    /// expression, or is a contract creation (`new C(args)`), so the caller
    /// falls through.
    pub fn try_emit_new_array(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        let Expression::NewExpression(new_expression) = call.operand() else {
            return Ok(None);
        };

        // slang types the array forms' call (`new T[]`); `new bytes` / `new
        // string` surface no call type, so recover them from the syntactic
        // elementary type name (both lower to a memory string).
        let builder = &self.expression_emitter.state.builder;
        let result_type = match call.get_type() {
            Some(inner @ (Type::Array(_) | Type::Bytes(_) | Type::String(_))) => {
                TypeConversion::resolve_slang_type(&inner, Some(DataLocation::Memory), builder)
            }
            None if matches!(new_expression.type_name(), TypeName::ElementaryType(_)) => {
                builder.types.string(DataLocation::Memory)
            }
            _ => return Ok(None),
        };

        let (values, block) = self.emit_argument_values(arguments, block)?;
        let builder = &self.expression_emitter.state.builder;
        let address = match values.first() {
            Some(&size_value) => {
                let size = TypeConversion::from_target_type(builder.types.ui256, builder)
                    .emit(size_value, builder, &block);
                builder.emit_sol_malloc_sized_zeroed(result_type, size, &block)
            }
            None => builder.emit_sol_malloc_zeroed(result_type, &block),
        };
        Ok(Some((Some(address), block)))
    }
}
