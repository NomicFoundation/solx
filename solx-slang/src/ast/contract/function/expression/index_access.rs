//!
//! Index access expression lowering: `a[i]`, `m[k]`, `s[i]`.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::r#type::TypeLike;
use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
use slang_solidity_v2::ast::IndexAccessExpression;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::ods::sol::LengthOperation;
use solx_mlir::ods::sol::SliceOperation;
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers `a[i]` / `m[k]` for arrays, dynamic `bytes`, mappings, and
    /// strings to `sol.gep` (sequential containers) or `sol.map` (mappings),
    /// followed by a `sol.load` of the addressed element. For dynamic
    /// `bytes` the C++ element type is `!sol.byte`; a `sol.bytes_cast`
    /// widens it to `!sol.fixedbytes<1>` to match Solidity's `bytes1`
    /// typing. `sol.bytes_cast` is a no-op for matching types.
    pub fn emit_index_access(
        &self,
        index_access: &IndexAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        // A slice `a[start:end]` produces a sub-array VALUE (not an element
        // address), lowered to `sol.slice`. `is_slice()` (the colon-presence
        // flag) distinguishes every slice form — including the open-ended
        // `a[i:]`, indistinguishable from the index `a[i]` by `end()` alone
        // (both `None`) — from a plain index access.
        if index_access.is_slice() {
            return self
                .emit_slice(index_access, block)
                .map(|(value, block)| (Some(value), block));
        }
        let (address, element_type, block) = self.emit_index_access_address(index_access, block)?;
        let value = self
            .state
            .builder
            .emit_sol_load(address, element_type, &block)?;
        // A scalar element loaded from a packed slot may need a fixed-bytes
        // re-alignment toward its declared element type (`sol.bytes_cast`). A
        // reference-typed element (a nested array / struct) is loaded as its
        // canonical reference and is authoritative: bytes_cast is undefined on
        // it, and slang can mis-type the *result* of indexing an array literal
        // whose element is a calldata reference (`[b[i:j]][0]`) as `calldata`
        // while the loaded value is the correct memory reference.
        if solx_mlir::TypeFactory::is_sol_reference(value.r#type()) {
            return Ok((Some(value), block));
        }
        let result_type = index_access
            .get_type()
            .expect("slang types every index-access expression");
        let slang_expected =
            TypeConversion::resolve_slang_type(&result_type, None, &self.state.builder);
        let value = self
            .state
            .builder
            .emit_sol_bytes_cast(value, slang_expected, &block);
        Ok((Some(value), block))
    }

    /// Emits a bounded calldata slice `a[start:end]` as a `sol.slice` value.
    /// `start` defaults to `0` when omitted (`a[:end]`); both indices are
    /// widened to `ui256`. Dispatched only when `end()` is `Some` (see
    /// [`Self::emit_index_access`]), so the upper bound is always explicit.
    fn emit_slice(
        &self,
        index_access: &IndexAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let base = index_access.operand();
        let (base_value, block) = self.emit_value(&base, block)?;
        let ui256 = self.state.builder.types.ui256;
        let (start_value, block) = match index_access.start() {
            Some(start_expression) => {
                let (value, block) = self.emit_value(&start_expression, block)?;
                let value = TypeConversion::coerce(value, ui256, &self.state.builder, &block);
                (value, block)
            }
            None => {
                let zero = self.state.builder.emit_sol_constant(0, ui256, &block);
                (zero, block)
            }
        };
        let (end_value, block) = match index_access.end() {
            Some(end_expression) => {
                let (value, block) = self.emit_value(&end_expression, block)?;
                let value = TypeConversion::coerce(value, ui256, &self.state.builder, &block);
                (value, block)
            }
            None => {
                // Open-ended slice `a[start:]` runs to the end of the array; its
                // upper bound is the operand's length.
                let builder = &self.state.builder;
                let length = sol_op!(builder, block, LengthOperation.inp(base_value).len(ui256));
                (length, block)
            }
        };
        let result_type = TypeConversion::resolve_slang_type(
            &index_access
                .get_type()
                .expect("slang types every slice expression"),
            None,
            &self.state.builder,
        );
        let builder = &self.state.builder;
        let value = sol_op!(
            builder,
            block,
            SliceOperation
                .arr(base_value)
                .start(start_value)
                .end(end_value)
                .res(result_type)
        );
        Ok((value, block))
    }

    /// Emits the address yielded by `a[i]` / `m[k]` together with the element
    /// MLIR type, without the trailing `sol.load`.
    ///
    /// Shared between the value-producing read path
    /// ([`Self::emit_index_access`]) and the lvalue write path in
    /// `emit_assignment`.
    pub fn emit_index_access_address(
        &self,
        index_access: &IndexAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        Value<'context, 'block>,
        Type<'context>,
        BlockRef<'context, 'block>,
    )> {
        if index_access.end().is_some() {
            unimplemented!("range index (a[i:j]) is not yet supported");
        }

        let base = index_access.operand();
        let index_expression = index_access
            .start()
            .expect("slang validates a[i] has an index expression");
        let base_type = base
            .get_type()
            .expect("slang types the base of an index access");
        let result_type = index_access
            .get_type()
            .expect("slang types every index-access expression");

        let (base_value, block) = self.emit_value(&base, block)?;
        let (index_value, block) = self.emit_value(&index_expression, block)?;

        let (address, element_type) = match &base_type {
            SlangType::Mapping(_) => {
                let element_type =
                    TypeConversion::resolve_slang_type(&result_type, None, &self.state.builder);
                let base_location = Self::resolve_base_location(&base_type);
                let address_type = Self::address_type(
                    &self.state.builder,
                    element_type,
                    base_location,
                    &result_type,
                );
                let address =
                    self.state
                        .builder
                        .emit_sol_map(base_value, index_value, address_type, &block);
                (address, element_type)
            }
            _ => {
                // SAFETY: `mlirSolGetEltType` returns a valid MlirType from
                // `sol::getEltType` on the C++ side; the struct-field index
                // is ignored for non-struct base types.
                let element_type = unsafe {
                    Type::from_raw(solx_mlir::ffi::mlirSolGetEltType(
                        base_value.r#type().to_raw(),
                        0,
                    ))
                };
                let address =
                    self.state
                        .builder
                        .emit_sol_gep(base_value, index_value, element_type, &block);
                (address, element_type)
            }
        };
        Ok((address, element_type, block))
    }

    /// Maps a slang container type's data location to the dialect-side
    /// `DataLocation`.
    fn resolve_base_location(base_type: &SlangType) -> DataLocation {
        match base_type.data_location() {
            Some(SlangDataLocation::Inherited) => {
                unreachable!("slang should not surface Inherited at an index-access base")
            }
            Some(other) => DataLocation::from_slang(other, None),
            None => unimplemented!(
                "index access on a value-typed base is not yet wired: {:?}",
                std::mem::discriminant(base_type)
            ),
        }
    }
}
