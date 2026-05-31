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

use solx_mlir::ods::sol::SliceOperation;
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

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
        // Calldata slice `a[start:end]` produces a sub-array VALUE (not an
        // element address), lowered to `sol.slice`. Only the explicit-end forms
        // (`a[i:j]`, `a[:j]`) are unambiguously slices in the AST; `a[i:]` is
        // indistinguishable from the index `a[i]` (both are `end == None`), so
        // it falls through to the element-access path below.
        if index_access.end().is_some() {
            return self
                .emit_slice(index_access, block)
                .map(|(value, block)| (Some(value), block));
        }
        let (address, element_type, block) = self.emit_index_access_address(index_access, block)?;
        let value = self
            .state
            .builder
            .emit_sol_load(address, element_type, &block)?;
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

    /// Emits a calldata slice `a[start:end]` as a `sol.slice` value. `start`
    /// defaults to `0` when omitted (`a[:j]`); `end` is always present here
    /// because the caller only routes the `end == Some` forms through this
    /// path. Indices are widened to `ui256`.
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
                let value = TypeConversion::from_target_type(ui256, &self.state.builder)
                    .emit(value, &self.state.builder, &block);
                (value, block)
            }
            None => {
                let zero = self.state.builder.emit_sol_constant(0, ui256, &block);
                (zero, block)
            }
        };
        let end_expression = index_access
            .end()
            .expect("emit_slice is only reached when the slice has an explicit end");
        let (end_value, block) = self.emit_value(&end_expression, block)?;
        let end_value = TypeConversion::from_target_type(ui256, &self.state.builder)
            .emit(end_value, &self.state.builder, &block);
        let result_type = TypeConversion::resolve_slang_type(
            &index_access
                .get_type()
                .expect("slang types every slice expression"),
            None,
            &self.state.builder,
        );
        let builder = &self.state.builder;
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
            // The value path (`emit_index_access`) intercepts slices; reaching
            // here means a slice was used as an assignment target, which is
            // not valid Solidity (calldata slices are read-only).
            anyhow::bail!("a calldata slice `a[i:j]` is not assignable");
        }

        let base = index_access.operand();
        let index_expression = index_access
            .start()
            .expect("slang validates a[i] has an index expression");
        let base_type = base
            .get_type()
            .ok_or_else(|| anyhow::anyhow!("base of index access has no resolved type"))?;
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
