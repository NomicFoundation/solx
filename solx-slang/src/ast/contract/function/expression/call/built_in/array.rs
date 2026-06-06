//!
//! Dynamic-array and `bytes` member built-ins: `push` and `pop`.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::Type as SlangType;

use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::type_conversion::TypeConversion;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Emits `arr.pop()` / `bytes.pop()` as `sol.pop`.
    pub fn emit_array_pop(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (array_value, block) = self
            .expression_emitter
            .emit_value(&access.operand(), block)?;
        self.expression_emitter
            .state
            .builder
            .emit_sol_pop(array_value, &block);
        Ok((None, block))
    }

    /// Emits `arr.push(x)` / `arr.push()` / `bytes.push()` as `sol.push`,
    /// followed by `sol.store` of the cast value when one is provided.
    /// Returns the new slot reference for the no-arg form, otherwise `None`.
    pub fn emit_array_push(
        &self,
        access: &MemberAccessExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let base = access.operand();
        let base_slang_type = base
            .get_type()
            .expect("slang types the base of an array push");
        let value_argument = arguments.iter().next();
        if value_argument.is_some() && matches!(&base_slang_type, SlangType::Bytes(_)) {
            unimplemented!("bytes.push(x) lowers to sol.push_string, which is not yet wired");
        }
        let builder = &self.expression_emitter.state.builder;

        let (element_type, slang_location) = match &base_slang_type {
            SlangType::Array(array_type) => (
                TypeConversion::resolve_slang_type(&array_type.element_type(), None, builder),
                array_type.location(),
            ),
            SlangType::Bytes(bytes_type) => (builder.types.fixed_bytes(1), bytes_type.location()),
            other => unreachable!(
                "Solidity's .push is a member of dynamic arrays and bytes only; got {:?}",
                std::mem::discriminant(other)
            ),
        };
        let base_location = match slang_location {
            SlangDataLocation::Inherited => {
                unreachable!("slang's binder should not surface Inherited at an array push base")
            }
            other => solx_utils::DataLocation::from_slang(other, None),
        };

        let (array_value, block) = self.expression_emitter.emit_value(&base, block)?;
        let address_type = builder.types.pointer(element_type, base_location);
        let new_slot = builder.emit_sol_push(array_value, address_type, &block);

        let Some(value_argument) = value_argument else {
            return Ok((Some(new_slot), block));
        };
        let (value, block) = self.expression_emitter.emit_value(&value_argument, block)?;
        let cast_value =
            TypeConversion::from_target_type(element_type, builder).emit(value, builder, &block);
        builder.emit_sol_store(cast_value, new_slot, &block);
        Ok((None, block))
    }
}
