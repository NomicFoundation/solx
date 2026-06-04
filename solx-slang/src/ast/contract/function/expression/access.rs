//!
//! Index access expression lowering: `a[i]`, `m[k]`.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
use slang_solidity_v2::ast::IndexAccessExpression;
use slang_solidity_v2::ast::Type as SlangType;
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

    /// Emits the address of `a[i]` / `m[k]` together with the element type,
    /// without the trailing load. Shared by the value read and the assignment
    /// lvalue path. Range indices (`a[i:j]`) defer to a later domain.
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
            unimplemented!("range index access (a[i:j])");
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
