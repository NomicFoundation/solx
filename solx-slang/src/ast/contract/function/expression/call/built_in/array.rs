//!
//! Dynamic-array and `bytes` member built-ins: `push` and `pop`.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
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
    /// followed by `sol.store` of the cast value when one is provided; returns
    /// the new slot reference for the no-arg form, otherwise `None`. The special
    /// case `bytes.push(x)` appends the byte in place via `sol.push_string`.
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
        if let (SlangType::Bytes(_), Some(value_argument)) = (&base_slang_type, &value_argument) {
            // `bytes.push(x)` appends a single byte in place via `sol.push_string`;
            // the packed element is not separately addressable, so unlike an array
            // push there is no returned slot to store into.
            let (bytes_reference, block) = self.expression_emitter.emit_value(&base, block)?;
            // `data.push("a")` appends a string-literal byte: materialise it as a
            // `byte` constant rather than a runtime `sol.string`.
            let byte_target = self.expression_emitter.state.builder.types.fixed_bytes(1);
            let (value, block) = self.expression_emitter.emit_value_for_target(
                value_argument,
                byte_target,
                block,
            )?;
            let builder = &self.expression_emitter.state.builder;
            let byte_value =
                TypeConversion::from_target_type(byte_target, builder).emit(value, builder, &block);
            builder.emit_sol_push_string(bytes_reference, byte_value, &block);
            return Ok((None, block));
        }
        let (new_slot, element_type, block) = self.emit_push_slot(access, block)?;
        let Some(value_argument) = value_argument else {
            // `arr.push()` in value position yields the freshly-appended element:
            // `sol.load` reads a value element as a fresh default and yields a
            // reference element as its canonical storage reference (the same dual
            // behaviour as an index access `a[i]`; the raw slot pointer would
            // mis-cast in the consumer).
            let builder = &self.expression_emitter.state.builder;
            let loaded = builder.emit_sol_load(new_slot, element_type, &block)?;
            return Ok((Some(loaded), block));
        };
        if solx_mlir::TypeFactory::is_sol_reference(element_type) {
            // A reference-typed element (nested array / struct / string) is
            // appended by copying the source (a memory aggregate) into the
            // storage slot `push` returns — the same memory→storage `sol.copy`
            // solc emits, and what the lvalue form `arr.push() = v` already does.
            let (value, block) = self.expression_emitter.emit_value(&value_argument, block)?;
            self.expression_emitter
                .state
                .builder
                .emit_sol_copy(value, new_slot, &block);
            return Ok((None, block));
        }
        let (value, block) =
            self.expression_emitter
                .emit_value_for_target(&value_argument, element_type, block)?;
        let builder = &self.expression_emitter.state.builder;
        let cast_value =
            TypeConversion::from_target_type(element_type, builder).emit(value, builder, &block);
        builder.emit_sol_store(cast_value, new_slot, &block);
        Ok((None, block))
    }

    /// Appends a default element to a dynamic array (or `bytes`) and returns the
    /// freshly-allocated slot reference together with its element MLIR type.
    /// Shared by `arr.push(x)` (which then stores `x` into the slot) and the
    /// `arr.push() = v` lvalue (which copies the right-hand side into it).
    pub fn emit_push_slot(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        Value<'context, 'block>,
        Type<'context>,
        BlockRef<'context, 'block>,
    )> {
        let base = access.operand();
        let base_slang_type = base
            .get_type()
            .expect("slang types the base of an array push");
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
        // solc's `sol.push` yields the new element's reference type directly when
        // the element is a reference type (nested array / struct / string) — the
        // slot is then copied into via `sol.copy` — and a `!sol.ptr` to the
        // element when it is a value type, stored into via `sol.store`. Mirror
        // that: a reference element pushed to a pointer would force a
        // memory→storage data-location cast the backend cannot lower.
        let push_result_type = if solx_mlir::TypeFactory::is_sol_reference(element_type) {
            element_type
        } else {
            builder.types.pointer(element_type, base_location)
        };
        let new_slot = builder.emit_sol_push(array_value, push_result_type, &block);
        Ok((new_slot, element_type, block))
    }
}
