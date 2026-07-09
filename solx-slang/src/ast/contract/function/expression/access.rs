//!
//! Index access expression lowering: `a[i]`, `m[k]`, `s[i]`.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
use slang_solidity_v2::ast::IndexAccessExpression;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Place;
use solx_mlir::Type;
use solx_mlir::Value;
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
    ///
    /// Fixed `bytesN` is a value, not an addressable container, so `bytesN[i]`
    /// extracts the byte directly with `sol.fixed_bytes_index` and never reaches
    /// the address path.
    pub fn emit_index_access(
        &self,
        index_access: &IndexAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let base = index_access.operand();
        let base_type = base
            .get_type()
            .ok_or_else(|| anyhow::anyhow!("base of index access has no resolved type"))?;

        if let SlangType::ByteArray(_) = base_type {
            let index_expression = index_access
                .start()
                .expect("slang validates a[i] has an index expression");
            let (base_value, block) = self.emit_value(&base, block)?;
            let (index_value, block) = self.emit_value(&index_expression, block)?;
            let value = base_value.fixed_bytes_index(index_value, self.state, &block);
            return Ok((Some(value), block));
        }

        let (address, element_type, block) = self.emit_index_access_address(index_access, block)?;
        let value = address.load(element_type, self.state, &block);
        let result_type = index_access
            .get_type()
            .expect("slang types every index-access expression");
        let slang_expected = TypeConversion::resolve_slang_type(&result_type, None, self.state);
        let value = value.bytes_cast(slang_expected, self.state, &block);
        Ok((Some(value), block))
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
        Place<'context, 'block>,
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
            .ok_or_else(|| anyhow::anyhow!("base of index access has no resolved type"))?;
        let result_type = index_access
            .get_type()
            .expect("slang types every index-access expression");

        let (base_value, block) = self.emit_value(&base, block)?;
        let (index_value, block) = self.emit_value(&index_expression, block)?;

        let (address, element_type) = match &base_type {
            SlangType::Mapping(_) => {
                let element_type =
                    TypeConversion::resolve_slang_type(&result_type, None, self.state);
                let base_location = Self::resolve_base_location(&base_type);
                let address_type =
                    Self::address_type(self.state, element_type, base_location, &result_type);
                let address =
                    Place::from(base_value).map(index_value, address_type, self.state, &block);
                (address, element_type)
            }
            _ => {
                let element_type = base_value.r#type().element_type(0);
                let address =
                    Place::from(base_value).gep(index_value, element_type, self.state, &block);
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
