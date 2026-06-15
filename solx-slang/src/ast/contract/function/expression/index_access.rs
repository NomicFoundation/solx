//!
//! Index access expression emission: `a[i]`, `m[k]`, `s[i]`.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::r#type::TypeLike;
use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
use slang_solidity_v2::ast::IndexAccessExpression;
use slang_solidity_v2::ast::IndexAccessKind;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::ods::sol::LengthOperation;
use solx_mlir::ods::sol::SliceOperation;
use solx_utils::DataLocation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::type_conversion::LocationPolicy;
use crate::ast::type_conversion::ResolveType;

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Emits the address yielded by `a[i]` / `m[k]` together with the element
    /// MLIR type, without the trailing `sol.load`.
    ///
    /// Shared between the value-producing read path (the `IndexAccessExpression`
    /// emission) and the lvalue write path in `emit_assignment`.
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

        let BlockAnd {
            value: base_value,
            block,
        } = base.emit(self, block)?;
        let BlockAnd {
            value: index_value,
            block,
        } = index_expression.emit(self, block)?;

        let (address, element_type) = match &base_type {
            SlangType::Mapping(_) => {
                let element_type =
                    result_type.resolve_type(LocationPolicy::Declared(None), &self.state.builder);
                let base_location = Self::resolve_base_location(&base_type);
                let address_type = Self::address_type(
                    &self.state.builder,
                    element_type,
                    base_location,
                    &result_type,
                );
                let address = base_value
                    .into_pointer()
                    .entry(
                        index_value,
                        crate::ast::Type::new(address_type),
                        &self.state.builder,
                        &block,
                    )
                    .into_mlir();
                (address, element_type)
            }
            _ => {
                // SAFETY: `mlirSolGetEltType` returns a valid MlirType from
                // `sol::getEltType` on the C++ side; the struct-field index
                // is ignored for non-struct base types.
                let element_type = unsafe {
                    Type::from_raw(solx_mlir::ffi::mlirSolGetEltType(
                        base_value.r#type().into_mlir().to_raw(),
                        0,
                    ))
                };
                let address = base_value
                    .into_pointer()
                    .gep(
                        index_value,
                        crate::ast::Type::new(element_type),
                        &self.state.builder,
                        &block,
                    )
                    .into_mlir();
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

// `a[i]` / `m[k]` for arrays, dynamic `bytes`, mappings, and strings address the
// element (`sol.gep` for sequential containers, `sol.map` for mappings) and
// `sol.load` it. For dynamic `bytes` the C++ element type is `!sol.byte`; a
// `sol.bytes_cast` widens it to `!sol.fixedbytes<1>` to match `bytes1` typing
// (a no-op for matching types). A slice `a[start:end]` instead produces a
// sub-array VALUE via `sol.slice`.
expression_emit!(IndexAccessExpression; |node, context, block| {
    // A slice `a[start:end]` produces a sub-array VALUE (not an element
    // address), emitted as `sol.slice`. `kind()` distinguishes every slice form
    // — including the open-ended `a[i:]`, indistinguishable from the index `a[i]`
    // by `end()` alone (both `None`) — from a plain index access. `start`
    // defaults to `0` when omitted (`a[:end]`); both indices widen to `ui256`.
    // The upper bound of an open-ended `a[start:]` is the operand's length.
    if matches!(node.kind(), IndexAccessKind::Slice) {
        let base = node.operand();
        let BlockAnd {
            value: base_value,
            block,
        } = base.emit(context, block)?;
        let ui256 =
            crate::ast::Type::unsigned(context.state.builder.context, solx_utils::BIT_LENGTH_FIELD)
                .into_mlir();
        let (start_value, block) = match node.start() {
            Some(start_expression) => {
                let BlockAnd { value, block } = start_expression.emit(context, block)?;
                let value = value
                    .coerce_to(crate::ast::Type::new(ui256), &context.state.builder, &block)
                    .into_mlir();
                (value, block)
            }
            None => {
                let zero = crate::ast::Value::constant(
                    0,
                    crate::ast::Type::new(ui256),
                    &context.state.builder,
                    &block,
                )
                .into_mlir();
                (zero, block)
            }
        };
        let (end_value, block) = match node.end() {
            Some(end_expression) => {
                let BlockAnd { value, block } = end_expression.emit(context, block)?;
                let value = value
                    .coerce_to(crate::ast::Type::new(ui256), &context.state.builder, &block)
                    .into_mlir();
                (value, block)
            }
            None => {
                let builder = &context.state.builder;
                let length = sol_op!(
                    builder,
                    block,
                    LengthOperation.inp(base_value.into_mlir()).len(ui256)
                );
                (length, block)
            }
        };
        let result_type = node
            .get_type()
            .expect("slang types every slice expression")
            .resolve_type(LocationPolicy::Declared(None), &context.state.builder);
        let builder = &context.state.builder;
        let value: Value<'context, 'block> = sol_op!(
            builder,
            block,
            SliceOperation
                .arr(base_value.into_mlir())
                .start(start_value)
                .end(end_value)
                .res(result_type)
        );
        return Ok(BlockAnd { block, value: value.into() });
    }
    let (address, element_type, block) = context.emit_index_access_address(node, block)?;
    let value = crate::ast::Pointer::new(address).load(
        crate::ast::Type::new(element_type),
        &context.state.builder,
        &block,
    );
    // A scalar element loaded from a packed slot may need a fixed-bytes
    // re-alignment toward its declared element type (`sol.bytes_cast`). A
    // reference-typed element (a nested array / struct) is loaded as its
    // canonical reference and is authoritative: bytes_cast is undefined on it,
    // and slang can mis-type the *result* of indexing an array literal whose
    // element is a calldata reference (`[b[i:j]][0]`) as `calldata` while the
    // loaded value is the correct memory reference.
    if value.r#type().is_reference() {
        return Ok(BlockAnd { block, value });
    }
    let result_type = node
        .get_type()
        .expect("slang types every index-access expression");
    let slang_expected =
        result_type.resolve_type(LocationPolicy::Declared(None), &context.state.builder);
    let value = value.cast(
        crate::ast::Type::new(slang_expected),
        &context.state.builder,
        &block,
    );
    Ok(BlockAnd { block, value })
});
