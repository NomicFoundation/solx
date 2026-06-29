//!
//! Index access expression emission: `a[i]`, `m[k]`, `s[i]`.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
use slang_solidity_v2::ast::IndexAccessExpression;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::ods::sol::SliceOperation;
use solx_utils::DataLocation;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::EmitPlace;
use crate::ast::LocationPolicy;
use crate::ast::Place;
use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;

impl<'context: 'block, 'block> EmitPlace<'context, 'block> for IndexAccessExpression {
    /// Emits the address `a[i]` / `m[k]` denotes with the element type, without the load.
    fn emit_place<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Place<'context, 'block>> {
        if self.end().is_some() {
            unreachable!("range index (a[i:j]) is not yet supported");
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
                let element_type =
                    AstType::resolve(&result_type, LocationPolicy::Declared(None), context.state);
                let base_location = match base_type.data_location() {
                    Some(SlangDataLocation::Inherited) => {
                        unreachable!("slang should not surface Inherited at an index-access base")
                    }
                    Some(other) => DataLocation::from_slang(other, None),
                    None => unreachable!(
                        "index access on a value-typed base is not yet wired: {:?}",
                        std::mem::discriminant(&base_type)
                    ),
                };
                let address_type =
                    AstType::new(element_type).address_type(base_location, context.state.mlir());
                let address = base_value
                    .into_pointer()
                    .entry(index_value, address_type, context.state, &block)
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
                        false,
                        context.state,
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

expression_emit!(IndexAccessExpression; |node, context, block| {
    if node.is_slice() {
        let base = node.operand();
        let BlockAnd {
            value: base_value,
            block,
        } = base.emit(context, block);
        let ui256 =
            AstType::unsigned(context.state.mlir(), solx_utils::BIT_LENGTH_FIELD)
                .into_mlir();
        let (start_value, block) = match node.start() {
            Some(start_expression) => {
                let BlockAnd { value, block } = start_expression.emit(context, block);
                let value = value
                    .cast(AstType::new(ui256), context.state, &block)
                    .into_mlir();
                (value, block)
            }
            None => {
                let zero = AstValue::constant(
                    0,
                    AstType::new(ui256),
                    context.state,
                    &block,
                )
                .into_mlir();
                (zero, block)
            }
        };
        let (end_value, block) = match node.end() {
            Some(end_expression) => {
                let BlockAnd { value, block } = end_expression.emit(context, block);
                let value = value
                    .cast(AstType::new(ui256), context.state, &block)
                    .into_mlir();
                (value, block)
            }
            None => {
                let length = base_value
                    .length(context.state, &block)
                    .into_mlir();
                (length, block)
            }
        };
        let result_type = AstType::resolve(
            &node
                .get_type()
                .expect("slang validated"),
            LocationPolicy::Declared(None),
            context.state,
        );
        let state = context.state;
        let value: Value<'context, 'block> = mlir_op!(
            state,
            block,
            SliceOperation
                .arr(base_value)
                .start(start_value)
                .end(end_value)
                .res(result_type)
        );
        return BlockAnd { block, value: value.into() };
    }
    let BlockAnd {
        value: Place {
            address,
            element_type,
        },
        block,
    } = node.emit_place(context, block);
    let value = Pointer::new(address).load(
        AstType::new(element_type),
        context.state,
        &block,
    );
    if value.r#type().is_reference() {
        return BlockAnd { block, value };
    }
    let result_type = node
        .get_type()
        .expect("slang validated");
    let slang_expected =
        AstType::resolve(&result_type, LocationPolicy::Declared(None), context.state);
    let value = value.cast(
        AstType::new(slang_expected),
        context.state,
        &block,
    );
    BlockAnd { block, value }
});
