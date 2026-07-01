//!
//! Index access expression lowering: `a[i]`, `m[k]`, `s[i]`.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::r#type::TypeLike;
use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
use slang_solidity_v2::ast::IndexAccessExpression;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_utils::DataLocation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_place::EmitPlace;
use crate::ast::place::Place;

impl<'context: 'block, 'block> EmitExpression<'context, 'block> for IndexAccessExpression {
    type Output = BlockAnd<'context, 'block, Value<'context, 'block>>;

    /// Lowers `a[i]` / `m[k]` for arrays, dynamic `bytes`, mappings, and
    /// strings to `sol.gep` (sequential containers) or `sol.map` (mappings),
    /// followed by a `sol.load` of the addressed element. For dynamic
    /// `bytes` the C++ element type is `!sol.byte`; a `sol.bytes_cast`
    /// widens it to `!sol.fixedbytes<1>` to match Solidity's `bytes1`
    /// typing. `sol.bytes_cast` is a no-op for matching types.
    fn emit<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> Self::Output {
        let BlockAnd {
            value: place,
            block,
        } = self.emit_place(context, block);
        let value =
            Pointer::new(place.address).load(AstType::new(place.element_type), context.state, &block);
        let result_type = self
            .get_type()
            .expect("slang types every index-access expression");
        let slang_expected =
            TypeConversion::resolve_slang_type(&result_type, None, context.state);
        let value = value
            .bytes_cast(AstType::new(slang_expected), context.state, &block)
            .into_mlir();
        BlockAnd { block, value }
    }
}

impl<'context: 'block, 'block> EmitPlace<'context, 'block> for IndexAccessExpression {
    /// Emits the address yielded by `a[i]` / `m[k]` together with the element
    /// MLIR type, without the trailing `sol.load`.
    ///
    /// Shared between the value-producing read path and the lvalue write path
    /// in `emit_assignment`.
    fn emit_place<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Place<'context, 'block>> {
        if self.end().is_some() {
            unimplemented!("range index (a[i:j]) is not yet supported");
        }

        let base = self.operand();
        let index_expression = self
            .start()
            .expect("slang validates a[i] has an index expression");
        let base_type = base
            .get_type()
            .expect("base of index access has a resolved type");
        let result_type = self
            .get_type()
            .expect("slang types every index-access expression");

        let BlockAnd {
            value: base_value,
            block,
        } = base.emit(context, block);
        let BlockAnd {
            value: index_value,
            block,
        } = index_expression.emit(context, block);

        let (address, element_type) = match &base_type {
            SlangType::Mapping(_) => {
                let element_type =
                    TypeConversion::resolve_slang_type(&result_type, None, context.state);
                let base_location = match base_type.data_location() {
                    Some(SlangDataLocation::Inherited) => {
                        unreachable!("slang should not surface Inherited at an index-access base")
                    }
                    Some(other) => DataLocation::from_slang(other, None),
                    None => unimplemented!(
                        "index access on a value-typed base is not yet wired: {:?}",
                        std::mem::discriminant(&base_type)
                    ),
                };
                let address_type = ExpressionContext::address_type(
                    context.state,
                    element_type,
                    base_location,
                    &result_type,
                );
                let address = Pointer::new(base_value)
                    .map(AstValue::new(index_value), AstType::new(address_type), context.state, &block)
                    .into_mlir();
                (address, element_type)
            }
            _ => {
                let element_type = unsafe {
                    Type::from_raw(solx_mlir::ffi::mlirSolGetEltType(
                        base_value.r#type().to_raw(),
                        0,
                    ))
                };
                let address = Pointer::new(base_value)
                    .gep(AstValue::new(index_value), AstType::new(element_type), context.state, &block)
                    .into_mlir();
                (address, element_type)
            }
        };
        BlockAnd {
            block,
            value: Place {
                address,
                element_type,
            },
        }
    }
}
