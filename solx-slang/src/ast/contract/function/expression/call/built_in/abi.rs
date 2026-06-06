//!
//! `abi.encode*` and `abi.decode` built-in lowering.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Operation;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::DenseI32ArrayAttribute;
use melior::ir::operation::OperationMutLike;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::Type as SlangType;
use solx_mlir::ods::sol::BytesCastOperation;
use solx_mlir::ods::sol::DecodeOperation;
use solx_mlir::ods::sol::EncodeOperation;

use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::built_in::EncodeMode;
use crate::ast::type_conversion::TypeConversion;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Emits `abi.encode(args)` as a standard `sol.encode`.
    pub fn emit_abi_encode(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (values, block) = self.emit_argument_values(arguments, block)?;
        let result = self.emit_sol_encode(&values, None, EncodeMode::Standard, &block);
        Ok((Some(result), block))
    }

    /// Emits `abi.encodePacked(args)` as a packed `sol.encode`.
    pub fn emit_abi_encode_packed(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (values, block) = self.emit_argument_values(arguments, block)?;
        let result = self.emit_sol_encode(&values, None, EncodeMode::Packed, &block);
        Ok((Some(result), block))
    }

    /// Emits `abi.encodeWithSelector(selector, args)`, casting the first
    /// argument to `!sol.fixed_bytes<4>` and prepending it to the payload.
    pub fn emit_abi_encode_with_selector(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (mut values, block) = self.emit_argument_values(arguments, block)?;
        let builder = &self.expression_emitter.state.builder;
        let selector =
            builder.emit_sol_cast(values.remove(0), builder.types.fixed_bytes(4), &block);
        let result = self.emit_sol_encode(&values, Some(selector), EncodeMode::Standard, &block);
        Ok((Some(result), block))
    }

    /// Emits `abi.encodeWithSignature("sig", args)`, hashing the literal
    /// signature to a 4-byte selector and prepending it to the payload.
    pub fn emit_abi_encode_with_signature(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let mut iter = arguments.iter();
        let signature_expression = iter.next().expect("slang validates non-empty arguments");
        let Expression::StringExpression(string_expression) = signature_expression else {
            unimplemented!(
                "abi.encodeWithSignature with a non-literal signature is not yet supported"
            );
        };
        let signature_bytes = string_expression.value();
        let hash = solx_utils::Keccak256Hash::from_slice(&signature_bytes);
        let selector_bytes: [u8; 4] = hash.as_bytes()[..4]
            .try_into()
            .expect("keccak256 always yields 32 bytes");
        let selector_word = u32::from_be_bytes(selector_bytes);
        let builder = &self.expression_emitter.state.builder;
        let selector_int = builder.emit_sol_constant(
            i64::from(selector_word),
            Type::from(IntegerType::unsigned(builder.context, 32)),
            &block,
        );
        let selector_value = block
            .append_operation(
                BytesCastOperation::builder(builder.context, builder.unknown_location)
                    .inp(selector_int)
                    .out(builder.types.fixed_bytes(4))
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.bytes_cast always produces one result")
            .into();
        let mut values = Vec::with_capacity(arguments.len() - 1);
        let mut current = block;
        for argument in iter {
            let (value, next) = self.expression_emitter.emit_value(&argument, current)?;
            values.push(value);
            current = next;
        }
        let result = self.emit_sol_encode(
            &values,
            Some(selector_value),
            EncodeMode::Standard,
            &current,
        );
        Ok((Some(result), current))
    }

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
    fn emit_sol_encode(
        &self,
        ins: &[Value<'context, 'block>],
        selector: Option<Value<'context, 'block>>,
        mode: EncodeMode,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let builder = &self.expression_emitter.state.builder;
        let mut op_builder = EncodeOperation::builder(builder.context, builder.unknown_location)
            .ins(ins)
            .res(builder.types.sol_string_memory);
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

    /// Emits `abi.decode(payload, (T))` as a `sol.decode` operation.
    ///
    /// The result type comes from the call's slang type (`call.get_type()`);
    /// multi-result decode requires the multi-result-call dispatch and is
    /// not yet supported.
    pub fn emit_abi_decode(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let payload_expression = arguments
            .iter()
            .next()
            .expect("slang validates the payload argument");
        let (payload_value, block) = self
            .expression_emitter
            .emit_value(&payload_expression, block)?;
        let return_slang_type = call
            .get_type()
            .expect("abi.decode call is typed by the binder");
        if matches!(return_slang_type, SlangType::Tuple(_)) {
            unimplemented!("abi.decode returning multiple values is not yet supported");
        }
        let builder = &self.expression_emitter.state.builder;
        let result_type = TypeConversion::resolve_slang_type(&return_slang_type, None, builder);
        let value = block
            .append_operation(
                DecodeOperation::builder(builder.context, builder.unknown_location)
                    .addr(payload_value)
                    .outs(&[result_type])
                    .build()
                    .into(),
            )
            .result(0)
            .expect("abi.decode single-result always produces one value")
            .into();
        Ok((value, block))
    }
}
