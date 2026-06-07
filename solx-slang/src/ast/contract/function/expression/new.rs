//!
//! `new` expression lowering: dynamic-aggregate allocation (`new T[](n)`,
//! `new bytes(n)`, `new string(n)`) and contract creation (`new C(args)`).
//!
//! An [`ExpressionEmitter`] method: `new.rs` lives in the expression module
//! subtree, so it lowers through the expression emitter directly rather than
//! the call emitter (the oracle's `built_in/new.rs` placement).
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::Type as SlangType;
use slang_solidity_v2::ast::TypeName as SlangTypeName;

use solx_utils::DataLocation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits a `new` expression: dynamic-aggregate allocation (`new T[](n)`,
    /// `new bytes(n)`) or contract creation (`new C(args)`).
    pub fn emit_new(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let slang_type = call.get_type();
        // `new T[](n)` / `new bytes(n)` / `new string(n)` allocate a dynamic
        // memory aggregate of `n` elements/bytes via a zeroed `sol.malloc`, the
        // count driving the length slot. slang resolves the array forms' call
        // type, but `new bytes` / `new string` surface no call type, so fall back
        // to the syntactic elementary type name (both lower to a memory string).
        let dynamic_result_type = match &slang_type {
            Some(inner @ (SlangType::Array(_) | SlangType::Bytes(_) | SlangType::String(_))) => {
                Some(TypeConversion::resolve_slang_type(
                    inner,
                    Some(DataLocation::Memory),
                    &self.state.builder,
                ))
            }
            None if matches!(
                call.operand(),
                Expression::NewExpression(new_expression)
                    if matches!(new_expression.type_name(), SlangTypeName::ElementaryType(_))
            ) =>
            {
                Some(self.state.builder.types.string(DataLocation::Memory))
            }
            _ => None,
        };
        if let Some(result_type) = dynamic_result_type {
            let mut values = Vec::with_capacity(arguments.len());
            let mut current_block = block;
            for argument in arguments.iter() {
                let (value, next) = self.emit_value(&argument, current_block)?;
                values.push(value);
                current_block = next;
            }
            let builder = &self.state.builder;
            let address =
                match values.first() {
                    Some(&size_value) => {
                        let size = TypeConversion::from_target_type(builder.types.ui256, builder)
                            .emit(size_value, builder, &current_block);
                        builder.emit_sol_malloc_sized_zeroed(result_type, size, &current_block)
                    }
                    None => builder.emit_sol_malloc_zeroed(result_type, &current_block),
                };
            return Ok((Some(address), current_block));
        }
        // Contract creation (`new C(args)` → `sol.new`) records a linker
        // dependency on the context, which is not yet tracked — a LOUD residual.
        unimplemented!("contract creation via `new` is not yet supported");
    }
}
