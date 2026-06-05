//!
//! Index access expression lowering: `a[i]`, `m[k]`.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
use slang_solidity_v2::ast::IndexAccessExpression;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::ods::sol::SliceOperation;
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers an index access `a[i]` / `m[k]` for arrays, dynamic `bytes`,
    /// strings, and mappings: address the element via `sol.gep` (sequential
    /// containers) or `sol.map` (mappings), then `sol.load` it. For dynamic
    /// `bytes` the element loads as `!sol.byte`; a trailing `sol.bytes_cast`
    /// widens it to the expression's `bytesN` type (a no-op for other types).
    pub fn emit_index_access(
        &self,
        index_access: &IndexAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        // A slice `a[start:end]` is a sub-array VALUE (not an element address),
        // lowered to `sol.slice`. `is_slice()` (the colon-presence flag) covers
        // every form — `a[i:j]`, `a[:j]`, and `a[i:]`, which is otherwise
        // indistinguishable from the index `a[i]` (both have `end() == None`).
        if index_access.is_slice() {
            return self.emit_slice(index_access, block);
        }
        let (address, element_type, block) = self.emit_index_access_address(index_access, block)?;
        let builder = &self.state.builder;
        let value = builder.emit_sol_load(address, element_type, &block)?;
        let result_type = index_access
            .get_type()
            .expect("the binder types every index-access expression");
        let expected_type = TypeConversion::resolve_slang_type(&result_type, None, builder);
        let value = builder.emit_sol_bytes_cast(value, expected_type, &block);
        Ok((value, block))
    }

    /// Emits a calldata slice `a[start:end]` as a `sol.slice` value. `start`
    /// defaults to `0` when omitted (`a[:j]`); `end` defaults to the operand's
    /// length when omitted (`a[i:]`). Both bounds are widened to `ui256`.
    fn emit_slice(
        &self,
        index_access: &IndexAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let base = index_access.operand();
        let (base_value, block) = self.emit_value(&base, block)?;
        let ui256 = self.state.builder.types.ui256;

        let (start_value, block) = match index_access.start() {
            Some(start) => {
                let (value, block) = self.emit_value(&start, block)?;
                let value = TypeConversion::from_target_type(ui256, &self.state.builder).emit(
                    value,
                    &self.state.builder,
                    &block,
                );
                (value, block)
            }
            None => (
                self.state.builder.emit_sol_constant(0, ui256, &block),
                block,
            ),
        };
        let (end_value, block) = match index_access.end() {
            Some(end) => {
                let (value, block) = self.emit_value(&end, block)?;
                let value = TypeConversion::from_target_type(ui256, &self.state.builder).emit(
                    value,
                    &self.state.builder,
                    &block,
                );
                (value, block)
            }
            // An open-ended slice `a[start:]` runs to the operand's length.
            None => (
                self.state.builder.emit_sol_length(base_value, &block),
                block,
            ),
        };

        let result_slang_type = index_access
            .get_type()
            .expect("the binder types every slice expression");
        let builder = &self.state.builder;
        let result_type = TypeConversion::resolve_slang_type(&result_slang_type, None, builder);
        let value = block
            .append_operation(
                SliceOperation::builder(builder.context, builder.unknown_location)
                    .arr(base_value)
                    .start(start_value)
                    .end(end_value)
                    .res(result_type)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.slice always produces one result")
            .into();
        Ok((value, block))
    }

    /// Emits the address of `a[i]` / `m[k]` together with the element type,
    /// without the trailing load. Shared by the value read and the assignment
    /// lvalue path. Slices (`a[i:j]`) are intercepted upstream as values, so one
    /// reaching this lvalue path is unreachable (a slice is not assignable).
    pub fn emit_index_access_address(
        &self,
        index_access: &IndexAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        Value<'context, 'block>,
        Type<'context>,
        BlockRef<'context, 'block>,
    )> {
        if index_access.is_slice() {
            // The value path (`emit_index_access`) intercepts slices; reaching
            // here means a slice is used as an assignment target, which is not
            // valid Solidity (calldata slices are read-only).
            unreachable!("a calldata slice `a[i:j]` is not an lvalue");
        }
        let base = index_access.operand();
        let index = index_access
            .start()
            .expect("the binder validates a[i] has an index expression");
        let base_type = base
            .get_type()
            .expect("the binder types every index-access base");

        let (base_value, block) = self.emit_value(&base, block)?;
        let (index_value, block) = self.emit_value(&index, block)?;
        let builder = &self.state.builder;

        if let SlangType::Mapping(_) = base_type {
            let result_type = index_access
                .get_type()
                .expect("the binder types every index-access expression");
            let element_type = TypeConversion::resolve_slang_type(&result_type, None, builder);
            let address_type = Self::address_type(
                builder,
                element_type,
                Self::base_location(&base_type),
                result_type.is_reference_type(),
            );
            let address = builder.emit_sol_map(base_value, index_value, address_type, &block);
            return Ok((address, element_type, block));
        }

        // Sequential containers (array / `bytes` / `string`) share one element
        // type, so the field index is irrelevant.
        let element_type = solx_mlir::TypeFactory::element_type(base_value.r#type(), 0);
        let address = builder.emit_sol_gep(base_value, index_value, element_type, &block);
        Ok((address, element_type, block))
    }

    /// The dialect address type for a mapping value: a reference-typed value in
    /// storage / calldata is addressed in place (the element type itself);
    /// every other value is reached through a `!sol.ptr<element, location>`.
    fn address_type(
        builder: &solx_mlir::Builder<'context>,
        element_type: Type<'context>,
        base_location: DataLocation,
        is_reference: bool,
    ) -> Type<'context> {
        if is_reference
            && matches!(
                base_location,
                DataLocation::Storage | DataLocation::CallData
            )
        {
            element_type
        } else {
            builder.types.pointer(element_type, base_location)
        }
    }

    /// Maps a container's Slang data location to its dialect [`DataLocation`].
    fn base_location(base_type: &SlangType) -> DataLocation {
        match base_type.data_location() {
            Some(SlangDataLocation::Inherited) => {
                unreachable!("an index-access base never carries an Inherited location")
            }
            Some(location) => DataLocation::from_slang(location, None),
            None => unimplemented!("index access on a value-typed base"),
        }
    }
}
