//!
//! Index access expression lowering: `a[i]`, `m[k]`, `s[i]`.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity::backend::ir::ast::IndexAccessExpression;
use slang_solidity::backend::ir::ast::Type as SlangType;
use slang_solidity::backend::types::DataLocation as SlangDataLocation;

use solx_utils::DataLocation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers `a[i]` / `m[k]` for arrays, dynamic `bytes`, mappings, and
    /// strings to `sol.gep` (sequential containers) or `sol.map` (mappings),
    /// followed by a `sol.load` of the addressed element.
    pub fn emit_index_access(
        &self,
        index_access: &IndexAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (address, element_type, block) = self.emit_index_access_address(index_access, block)?;
        let value = self
            .state
            .builder
            .emit_sol_load(address, element_type, &block)?;
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
        let base_slang_type = base
            .get_type()
            .ok_or_else(|| anyhow::anyhow!("base of index access has no resolved type"))?;
        let result_slang_type = index_access
            .get_type()
            .expect("slang types every index-access expression");

        let (base_value, block) = self.emit_value(&base, block)?;
        let (index_value, block) = self.emit_value(&index_expression, block)?;

        let element_type =
            TypeConversion::resolve_slang_type(&result_slang_type, None, &self.state.builder);
        let base_location = Self::resolve_base_location(&base_slang_type);
        let address_type = Self::address_type(
            &self.state.builder,
            element_type,
            base_location,
            &result_slang_type,
        );

        let address = match &base_slang_type {
            SlangType::Mapping(_) => {
                self.state
                    .builder
                    .emit_sol_map(base_value, index_value, address_type, &block)
            }
            _ => self
                .state
                .builder
                .emit_sol_gep(base_value, index_value, address_type, &block),
        };
        Ok((address, element_type, block))
    }

    /// Maps a slang container type's data location to the dialect-side
    /// `DataLocation`.
    fn resolve_base_location(base_slang_type: &SlangType) -> DataLocation {
        match base_slang_type.data_location() {
            Some(SlangDataLocation::Inherited) => {
                unreachable!("slang should not surface Inherited at an index-access base")
            }
            Some(other) => DataLocation::from_slang(other, None),
            None => unimplemented!(
                "index access on a value-typed base is not yet wired: {:?}",
                std::mem::discriminant(base_slang_type)
            ),
        }
    }
}
