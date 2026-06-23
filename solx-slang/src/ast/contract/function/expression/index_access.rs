//!
//! Index access expression emission: `a[i]`, `m[k]`, `s[i]`.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
use slang_solidity_v2::ast::IndexAccessExpression;
use slang_solidity_v2::ast::Type as SlangType;

use solx_utils::DataLocation;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::EmitPlace;
use crate::ast::LocationPolicy;
use crate::ast::Place;
use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::contract::function::expression::ExpressionContext;

impl<'context: 'block, 'block> EmitPlace<'context, 'block> for IndexAccessExpression {
    /// Emits the address `a[i]` / `m[k]` denotes with the element type (`sol.map` for a mapping key,
    /// `sol.gep` for a sequential index), without the load.
    fn emit_place<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Place<'context, 'block>> {
        if self.end().is_some() {
            unimplemented!("range index (a[i:j]) is not yet supported");
        }

        let base = self.operand();
        let index_expression = self.start().expect("slang validated");
        let base_type = base.get_type().expect("slang validated");
        let result_type = self.get_type().expect("slang validated");

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
                let element_type = AstType::resolve(
                    &result_type,
                    LocationPolicy::Declared(None),
                    &context.state.builder,
                );
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
                let address_type = AstType::new(element_type)
                    .address_type(base_location, context.state.builder.context);
                let address = base_value
                    .into_pointer()
                    .entry(index_value, address_type, &context.state.builder, &block)
                    .into_mlir();
                (address, element_type)
            }
            _ => {
                let element_type = base_value.r#type().element_type(0).into_mlir();
                let address = base_value
                    .into_pointer()
                    .gep(
                        index_value,
                        AstType::new(element_type),
                        &context.state.builder,
                        &block,
                    )
                    .into_mlir();
                (address, element_type)
            }
        };
        BlockAnd {
            value: Place {
                address,
                element_type,
            },
            block,
        }
    }
}

// `a[i]` / `m[k]` address the element and `sol.load` it.
expression_emit!(IndexAccessExpression; |node, context, block| {
    let BlockAnd {
        value: Place {
            address,
            element_type,
        },
        block,
    } = node.emit_place(context, block);
    let value = Pointer::new(address).load(
        AstType::new(element_type),
        &context.state.builder,
        &block,
    );
    // A loaded element is cast to its slang-declared type (a scalar may need a fixed-bytes
    // re-alignment).
    let result_type = node
        .get_type()
        .expect("slang validated");
    let slang_expected =
        AstType::resolve(&result_type, LocationPolicy::Declared(None), &context.state.builder);
    let value = value.cast(
        AstType::new(slang_expected),
        &context.state.builder,
        &block,
    );
    BlockAnd { block, value }
});
