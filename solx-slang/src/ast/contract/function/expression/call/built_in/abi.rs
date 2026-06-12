//!
//! `abi.encode*` and `abi.decode` built-in lowering.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Operation;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::attribute::DenseI32ArrayAttribute;
use melior::ir::operation::OperationMutLike;
use num_bigint::BigInt;
use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::Type as SlangType;
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

    /// Emits `abi.encodeWithSignature(sig, args)`, hashing the signature to a
    /// 4-byte selector and prepending it to the payload. A literal signature is
    /// hashed at compile time; a runtime one through `keccak256`.
    pub fn emit_abi_encode_with_signature(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let mut iter = arguments.iter();
        let signature_expression = iter.next().expect("slang validates non-empty arguments");
        // A literal signature hashes at compile time to a constant selector; a
        // runtime signature (`abi.encodeWithSignature(sig, …)`) is hashed by
        // `keccak256` and truncated to its leading four bytes.
        let (selector_value, mut current) = match &signature_expression {
            Expression::StringExpression(string_expression) => {
                let signature_bytes = string_expression.value();
                let hash = solx_utils::Keccak256Hash::from_slice(&signature_bytes);
                let selector_bytes: [u8; 4] = hash.as_bytes()[..4]
                    .try_into()
                    .expect("keccak256 always yields 32 bytes");
                let selector_word = u32::from_be_bytes(selector_bytes);
                let selector_value =
                    self.emit_selector_constant(&BigInt::from(selector_word), 4, &block);
                (selector_value, block)
            }
            _ => {
                let (signature_value, current) = self
                    .expression_emitter
                    .emit_value(&signature_expression, block)?;
                // The runtime signature is hashed by `keccak256` and truncated to
                // its leading four bytes.
                let hash = self.emit_keccak256(signature_value, &current);
                let builder = &self.expression_emitter.state.builder;
                let selector_value =
                    TypeConversion::coerce(hash, builder.types.fixed_bytes(4), builder, &current);
                (selector_value, current)
            }
        };
        let mut values = Vec::with_capacity(arguments.len() - 1);
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

    /// Emits `abi.encodeCall(callee, args)`: the callee's 4-byte selector
    /// prepended to its ABI-encoded arguments. A static function reference
    /// (`C.f`, `this.f`) folds its selector to a compile-time constant via
    /// `compute_selector`; a runtime function-pointer value (a state/local
    /// variable, a returned pointer) reads its selector at runtime via
    /// `sol.ext_func_selector`. The arguments are the second argument — a tuple
    /// `(a, b)` spread element-wise, or a single non-tuple value — coerced to the
    /// callee's declared parameter types, so an integer literal encodes at the
    /// parameter's width (matching solc). The callee is classified by resolving
    /// the reference to its definition / function-pointer type, never by name
    /// text (Rule-7).
    pub fn emit_abi_encode_call(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let mut iter = arguments.iter();
        let function_expression = iter
            .next()
            .expect("abi.encodeCall takes a function reference");
        let call_arguments = iter
            .next()
            .expect("abi.encodeCall takes a call-arguments argument");
        let definition = match &function_expression {
            Expression::MemberAccessExpression(access) => access.member().resolve_to_definition(),
            Expression::Identifier(identifier) => identifier.resolve_to_definition(),
            _ => None,
        };
        let builder = &self.expression_emitter.state.builder;
        // The selector and the parameter types the arguments are coerced to come
        // from either a static function reference — selector folded to a
        // compile-time constant, parameter types from the function definition —
        // or a runtime function-pointer value — selector read with
        // `sol.ext_func_selector`, parameter types from the pointer's declared
        // function type.
        let (selector_value, parameter_types, current) = match definition {
            Some(Definition::Function(function)) => {
                let selector = function
                    .compute_selector()
                    .expect("abi.encodeCall's callee is an external function with an ABI selector");
                let selector_value =
                    self.emit_selector_constant(&BigInt::from(selector), 4, &block);
                // `abi.encodeCall` ABI-encodes the arguments as an external call
                // would: reference parameters are encoded from `Memory`, not
                // their declared `calldata`/`storage` location (which cannot
                // cross the call boundary). Use the external (memory) signature
                // so a memory struct/array argument needs no data-location cast
                // (solc encodes the same).
                let (parameter_types, _) =
                    TypeConversion::resolve_external_function_types(&function, builder);
                (selector_value, parameter_types, block)
            }
            _ => {
                let (function_value, current) = self
                    .expression_emitter
                    .emit_value(&function_expression, block)?;
                assert!(
                    solx_mlir::TypeFactory::is_sol_ext_function_ref(function_value.r#type()),
                    "abi.encodeCall's runtime callee resolves to an external function pointer"
                );
                let selector_value = builder.emit_sol_ext_func_selector(function_value, &current);
                let SlangType::Function(function_type) = function_expression
                    .get_type()
                    .expect("slang types every function-pointer expression")
                else {
                    unreachable!("a non-static abi.encodeCall callee is a function pointer")
                };
                let parameter_types = function_type
                    .parameter_types()
                    .iter()
                    .map(|parameter_type| {
                        TypeConversion::resolve_slang_type_in_memory(parameter_type, builder)
                    })
                    .collect();
                (selector_value, parameter_types, current)
            }
        };
        // The call arguments are the second argument: a tuple spreads to one
        // value per element, a single non-tuple value is the sole argument.
        let argument_expressions: Vec<Expression> = match call_arguments {
            Expression::TupleExpression(tuple) => tuple
                .items()
                .iter()
                .filter_map(|item| item.expression())
                .collect(),
            other => vec![other],
        };
        let (values, current) = self.emit_coerced_argument_expressions(
            &argument_expressions,
            &parameter_types,
            current,
        )?;
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
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let payload_expression = arguments
            .iter()
            .next()
            .expect("slang validates the payload argument");
        let (payload_value, block) = self
            .expression_emitter
            .emit_value(&payload_expression, block)?;
        let result_types = self.abi_decode_result_types(call);
        let builder = &self.expression_emitter.state.builder;
        // `sol.decode` requires a memory or calldata byte buffer; a storage
        // `bytes` / `string` is a reference, so copy it to memory first (solc
        // emits a Storage->Memory `sol.data_loc_cast` here). Memory and calldata
        // payloads are already valid buffers and pass through unchanged.
        let payload_value = if matches!(
            payload_expression
                .get_type()
                .and_then(|payload_type| payload_type.data_location()),
            Some(SlangDataLocation::Storage)
        ) {
            TypeConversion::coerce(
                payload_value,
                builder.types.sol_string_memory,
                builder,
                &block,
            )
        } else {
            payload_value
        };
        let operation = block.append_operation(
            DecodeOperation::builder(builder.context, builder.unknown_location)
                .addr(payload_value)
                .outs(&result_types)
                .build()
                .into(),
        );
        let values = (0..result_types.len())
            .map(|index| {
                operation
                    .result(index)
                    .expect("sol.decode yields one result per requested type")
                    .into()
            })
            .collect();
        Ok((values, block))
    }

    /// The MLIR result types of an `abi.decode` call — one per requested type.
    /// `abi.decode(data, T)` yields one; `abi.decode(data, (A, B, …))` yields
    /// one per tuple element. Resolved from the call's binder-assigned type.
    fn abi_decode_result_types(&self, call: &FunctionCallExpression) -> Vec<Type<'context>> {
        let builder = &self.expression_emitter.state.builder;
        let return_slang_type = call
            .get_type()
            .expect("abi.decode call is typed by the binder");
        match return_slang_type {
            SlangType::Tuple(tuple) => tuple
                .types()
                .iter()
                .map(|slang_type| TypeConversion::resolve_slang_type(slang_type, None, builder))
                .collect(),
            other => vec![TypeConversion::resolve_slang_type(&other, None, builder)],
        }
    }
}
