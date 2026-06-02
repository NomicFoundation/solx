//!
//! Dynamic-array / `bytes` member built-ins: `arr.pop()` / `bytes.pop()` and
//! `arr.push(x)` / `arr.push()` / `bytes.push()`, plus the shared push-slot
//! helper (also used by the push-as-lvalue assignment path).
//!

use super::*;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Emits `arr.pop()` / `bytes.pop()` as `sol.pop`.
    pub(crate) fn emit_array_pop(
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
    pub(crate) fn emit_array_push(
        &self,
        access: &MemberAccessExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let base = access.operand();
        let base_slang_type = base
            .get_type()
            .ok_or_else(|| anyhow::anyhow!("base of array push has no resolved type"))?;
        let value_argument = arguments.iter().next();
        let builder = &self.expression_emitter.state.builder;

        // `bytes.push(x)` has a dedicated lowering (`sol.push_string`) that
        // handles the in-place → out-of-place encoding transition; the generic
        // `sol.push` reference path below is only for value-typed dynamic arrays
        // and the no-argument `bytes.push()` overload.
        if matches!(&base_slang_type, SlangType::Bytes(_))
            && let Some(push_value) = &value_argument
        {
            let (array_value, block) = self.expression_emitter.emit_value(&base, block)?;
            // `emit_value_for_target` materializes a string literal (`data.push("a")`)
            // as a fixedbytes constant rather than a memory string.
            let (value, block) = self
                .expression_emitter
                .emit_value_for_target(push_value, builder.types.fixed_bytes(1), block)?;
            let byte_value = TypeConversion::from_target_type(builder.types.fixed_bytes(1), builder)
                .emit(value, builder, &block);
            builder.emit_sol_push_string(array_value, byte_value, &block);
            return Ok((None, block));
        }

        let (new_slot, element_type, block) = self.emit_push_slot(access, block)?;

        let Some(value_argument) = value_argument else {
            // `arr.push()` in value position yields the freshly-appended element.
            // A value element (`uint[].push()`) is loaded from the slot (a fresh
            // default); a reference element (`uint[][].push()`) is the slot
            // reference itself, used to initialise a storage pointer.
            let builder = &self.expression_emitter.state.builder;
            if IntegerType::try_from(element_type).is_ok() {
                let loaded = builder.emit_sol_load(new_slot, element_type, &block)?;
                return Ok((Some(loaded), block));
            }
            return Ok((Some(new_slot), block));
        };
        let (value, block) = self.expression_emitter.emit_value(&value_argument, block)?;
        let builder = &self.expression_emitter.state.builder;
        let cast_value =
            TypeConversion::from_target_type(element_type, builder).emit(value, builder, &block);
        builder.emit_sol_store(cast_value, new_slot, &block);
        Ok((None, block))
    }

    /// Emits `sol.push` for `arr.push()` / `bytes.push()`, returning the new
    /// element's reference, its element type, and the continued block. Shared by
    /// the value-returning push and the push-as-lvalue (`arr.push() = v`), where
    /// the caller stores the right-hand side into the returned reference.
    pub(crate) fn emit_push_slot(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, Type<'context>, BlockRef<'context, 'block>)> {
        let base = access.operand();
        let base_slang_type = base
            .get_type()
            .ok_or_else(|| anyhow::anyhow!("base of array push has no resolved type"))?;
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
        Ok((new_slot, element_type, block))
    }
}
