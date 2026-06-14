//!
//! Dynamic-array and `bytes` member built-ins: `push` and `pop`.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::Type as SlangType;
use solx_mlir::ods::sol::CopyOperation;
use solx_mlir::ods::sol::PopOperation;
use solx_mlir::ods::sol::PushOperation;
use solx_mlir::ods::sol::PushStringOperation;
use solx_mlir::ods::sol::StoreOperation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::Toward;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::type_conversion::LocationPolicy;
use crate::ast::type_conversion::ResolveType;

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Emits `arr.pop()` / `bytes.pop()` as `sol.pop`.
    pub fn emit_array_pop(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let BlockAnd {
            value: array_value,
            block,
        } = access.operand().emit(self, block)?;
        sol_op_void!(
            &self.state.builder,
            &block,
            PopOperation.inp(array_value.into_mlir())
        );
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
            let BlockAnd {
                value: bytes_reference,
                block,
            } = base.emit(self, block)?;
            // `data.push("a")` appends a string-literal byte: materialise it as a
            // `byte` constant rather than a runtime `sol.string`.
            let byte_target =
                crate::ast::Type::fixed_bytes(self.state.builder.context, 1).into_mlir();
            let BlockAnd { value, block } = (Toward {
                expression: value_argument,
                target_type: byte_target,
            })
            .emit(self, block)?;
            let builder = &self.state.builder;
            let byte_value = value
                .coerce_to(crate::ast::Type::new(byte_target), builder, &block)
                .into_mlir();
            sol_op_void!(
                builder,
                &block,
                PushStringOperation
                    .addr(bytes_reference.into_mlir())
                    .value(byte_value)
            );
            return Ok((None, block));
        }
        let (new_slot, element_type, block) = self.emit_push_slot(access, block)?;
        let Some(value_argument) = value_argument else {
            // `arr.push()` in value position yields the freshly-appended element:
            // `sol.load` reads a value element as a fresh default and yields a
            // reference element as its canonical storage reference (the same dual
            // behaviour as an index access `a[i]`; the raw slot pointer would
            // mis-cast in the consumer).
            let builder = &self.state.builder;
            let loaded = builder.emit_sol_load(new_slot, element_type, &block)?;
            return Ok((Some(loaded), block));
        };
        if crate::ast::Type::new(element_type).is_reference() {
            // A reference-typed element (nested array / struct / string) is
            // appended by copying the source (a memory aggregate) into the
            // storage slot `push` returns — the same memory→storage `sol.copy`
            // solc emits, and what the lvalue form `arr.push() = v` already does.
            let BlockAnd { value, block } = value_argument.emit(self, block)?;
            sol_op_void!(
                &self.state.builder,
                &block,
                CopyOperation.src(value.into_mlir()).dst(new_slot)
            );
            return Ok((None, block));
        }
        let BlockAnd { value, block } = (Toward {
            expression: &value_argument,
            target_type: element_type,
        })
        .emit(self, block)?;
        let builder = &self.state.builder;
        let cast_value = value
            .coerce_to(crate::ast::Type::new(element_type), builder, &block)
            .into_mlir();
        sol_op_void!(
            builder,
            &block,
            StoreOperation.val(cast_value).addr(new_slot)
        );
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
        let builder = &self.state.builder;
        let (element_type, slang_location) = match &base_slang_type {
            SlangType::Array(array_type) => (
                array_type
                    .element_type()
                    .resolve_type(LocationPolicy::Declared(None), builder),
                array_type.location(),
            ),
            SlangType::Bytes(bytes_type) => (
                crate::ast::Type::fixed_bytes(builder.context, 1).into_mlir(),
                bytes_type.location(),
            ),
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
        let BlockAnd {
            value: array_value,
            block,
        } = base.emit(self, block)?;
        // solc's `sol.push` yields the new element's reference type directly when
        // the element is a reference type (nested array / struct / string) — the
        // slot is then copied into via `sol.copy` — and a `!sol.ptr` to the
        // element when it is a value type, stored into via `sol.store`. Mirror
        // that: a reference element pushed to a pointer would force a
        // memory→storage data-location cast the backend cannot lower.
        let push_result_type = if crate::ast::Type::new(element_type).is_reference() {
            element_type
        } else {
            crate::ast::Type::pointer(builder.context, element_type, base_location).into_mlir()
        };
        let new_slot = sol_op!(
            builder,
            &block,
            PushOperation
                .inp(array_value.into_mlir())
                .addr(push_result_type)
        );
        Ok((new_slot, element_type, block))
    }
}
