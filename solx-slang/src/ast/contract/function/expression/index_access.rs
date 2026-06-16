//!
//! Index access expression emission: `a[i]`, `m[k]`, `s[i]`.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
use slang_solidity_v2::ast::IndexAccessExpression;
use slang_solidity_v2::ast::IndexAccessKind;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::ods::sol::LengthOperation;
use solx_mlir::ods::sol::SliceOperation;
use solx_utils::DataLocation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::EmitAddress;
use crate::ast::LocationPolicy;
use crate::ast::Place;
use crate::ast::contract::function::expression::ExpressionContext;

impl<'state, 'context, 'block, 'scope> EmitAddress<'context, 'block, 'state, 'scope>
    for IndexAccessExpression
where
    'context: 'block,
    'context: 'state,
    'block: 'state,
    'state: 'scope,
{
    type Context = &'scope ExpressionContext<'state, 'context, 'block>;

    /// Emits the address `a[i]` / `m[k]` denotes together with the element MLIR
    /// type — `sol.map` over a mapping key, `sol.gep` over a sequential index —
    /// without the trailing `sol.load`.
    fn emit_address(
        &self,
        context: Self::Context,
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
            .expect("slang types the base of an index access");
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
                let element_type = crate::ast::Type::resolve(
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
                let address_type = crate::ast::Type::new(element_type)
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
                        crate::ast::Type::new(element_type),
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
        } = base.emit(context, block);
        let ui256 =
            crate::ast::Type::unsigned(context.state.builder.context, solx_utils::BIT_LENGTH_FIELD)
                .into_mlir();
        let (start_value, block) = match node.start() {
            Some(start_expression) => {
                let BlockAnd { value, block } = start_expression.emit(context, block);
                let value = value
                    .cast(crate::ast::Type::new(ui256), &context.state.builder, &block)
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
                let BlockAnd { value, block } = end_expression.emit(context, block);
                let value = value
                    .cast(crate::ast::Type::new(ui256), &context.state.builder, &block)
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
        let result_type = crate::ast::Type::resolve(
            &node
                .get_type()
                .expect("slang types every slice expression"),
            LocationPolicy::Declared(None),
            &context.state.builder,
        );
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
        return BlockAnd { block, value: value.into() };
    }
    let BlockAnd {
        value: Place {
            address,
            element_type,
        },
        block,
    } = node.emit_address(context, block);
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
        return BlockAnd { block, value };
    }
    let result_type = node
        .get_type()
        .expect("slang types every index-access expression");
    let slang_expected =
        crate::ast::Type::resolve(&result_type, LocationPolicy::Declared(None), &context.state.builder);
    let value = value.cast(
        crate::ast::Type::new(slang_expected),
        &context.state.builder,
        &block,
    );
    BlockAnd { block, value }
});
