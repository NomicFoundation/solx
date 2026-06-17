//!
//! The shared ABI primitives the `abi.encode*` / `abi.decode` arms reach: the
//! `sol.encode` op constructor and the `abi.decode` result-type resolution.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Operation;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::DenseI32ArrayAttribute;
use melior::ir::operation::OperationMutLike;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::Type as SlangType;
use solx_mlir::ods::sol::EncodeOperation;

use crate::ast::LocationPolicy;
use crate::ast::Type as AstType;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::built_in::EncodeMode;

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Emits a `sol.encode` operation producing a `bytes memory` payload.
    ///
    /// `selector`, when present, is prepended as the first 4 bytes of the
    /// payload and must already be of `!sol.fixed_bytes<4>` type.
    /// [`EncodeMode::Packed`] emits the ABI-packed encoding (no per-element
    /// padding).
    ///
    /// Sets `operand_segment_sizes` manually because melior's ODS-generated
    /// builder does not synthesize the attribute for `AttrSizedOperandSegments`
    /// ops; the dialect verifier rejects the op without it.
    pub fn emit_sol_encode(
        &self,
        ins: &[Value<'context, 'block>],
        selector: Option<Value<'context, 'block>>,
        mode: EncodeMode,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let builder = &self.state.builder;
        let mut op_builder = EncodeOperation::builder(builder.context, builder.unknown_location)
            .ins(ins)
            .res(AstType::string(builder.context, solx_utils::DataLocation::Memory).into_mlir());
        if let Some(selector_value) = selector {
            op_builder = op_builder.selector(selector_value);
        }
        if matches!(mode, EncodeMode::Packed) {
            op_builder = op_builder.packed(Attribute::unit(builder.context));
        }
        let mut operation: Operation = op_builder.build().into();
        // TODO: drop this manual segment-sizes plumbing once the melior op-builder
        // macro emits `operand_segment_sizes` automatically for ops with variadic
        // or optional operand groups.
        let ins_count = i32::try_from(ins.len()).expect("encode argument count fits in i32");
        let segment_sizes = DenseI32ArrayAttribute::new(
            builder.context,
            &[ins_count, i32::from(selector.is_some())],
        );
        operation.set_inherent_attribute("operand_segment_sizes", segment_sizes.into());
        block
            .append_operation(operation)
            .result(0)
            .expect("sol.encode always produces one result")
            .into()
    }

    /// The MLIR result types of an `abi.decode` call — one per requested type.
    /// `abi.decode(data, T)` yields one; `abi.decode(data, (A, B, …))` yields
    /// one per tuple element. Resolved from the call's binder-assigned type.
    pub fn abi_decode_result_types(&self, call: &FunctionCallExpression) -> Vec<Type<'context>> {
        let builder = &self.state.builder;
        let return_slang_type = call.get_type().expect("slang validated");
        match return_slang_type {
            SlangType::Tuple(tuple) => tuple
                .types()
                .iter()
                .map(|slang_type| {
                    AstType::resolve(slang_type, LocationPolicy::Declared(None), builder)
                })
                .collect(),
            other => vec![AstType::resolve(
                &other,
                LocationPolicy::Declared(None),
                builder,
            )],
        }
    }
}
