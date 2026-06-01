//!
//! ABI encode/decode (`abi.encode*` / `abi.decode`) lowering.
//!

use super::*;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Emits a `sol.encode` operation producing a `bytes memory` payload.
    ///
    /// `selector`, when present, is prepended as the first 4 bytes of the
    /// payload and must already be of `!sol.fixed_bytes<4>` type. `packed`
    /// emits the ABI-packed encoding (no per-element padding).
    ///
    /// Sets `operand_segment_sizes` manually because melior's ODS-generated
    /// builder does not synthesize the attribute for `AttrSizedOperandSegments`
    /// ops; the dialect verifier rejects the op without it.
    pub(super) fn emit_sol_encode(
        &self,
        ins: &[Value<'context, 'block>],
        selector: Option<Value<'context, 'block>>,
        packed: bool,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let builder = &self.expression_emitter.state.builder;
        let mut op_builder = EncodeOperation::builder(builder.context, builder.unknown_location)
            .ins(ins)
            .res(builder.types.sol_string_memory);
        if let Some(selector_value) = selector {
            op_builder = op_builder.selector(selector_value);
        }
        if packed {
            op_builder = op_builder.packed(Attribute::unit(builder.context));
        }
        // `operand_segment_sizes` ([ins.len(), selector?]) is synthesized by the
        // melior op-builder macro for this `AttrSizedOperandSegments` op.
        let operation: Operation = op_builder.build().into();
        block
            .append_operation(operation)
            .result(0)
            .expect("sol.encode always produces one result")
            .into()
    }

    /// Resolves an `abi.decode` type-list element — a type name in value
    /// position such as `uint256`, `bytes`, or `bytes32` — to its MLIR type.
    ///
    /// Slang does not assign a semantic type to a type name used as an
    /// expression (its `get_type()` is `None`, and the enclosing `abi.decode`
    /// call is typed as a tuple of `Void`), so the type is reconstructed
    /// structurally. Only elementary types are supported; arrays, mappings,
    /// and user-defined types bail so the decode falls back to failing rather
    /// than producing a wrong type.
    fn resolve_abi_type_expression(
        &self,
        expression: &Expression,
    ) -> anyhow::Result<Type<'context>> {
        match expression {
            Expression::ElementaryType(elementary) => {
                self.resolve_abi_elementary_type(elementary)
            }
            Expression::TypeExpression(type_expression) => {
                match type_expression.type_name() {
                    SlangTypeName::ElementaryType(elementary) => {
                        self.resolve_abi_elementary_type(&elementary)
                    }
                    _ => anyhow::bail!(
                        "abi.decode of arrays, mappings, and user-defined types is not yet supported"
                    ),
                }
            }
            _ => anyhow::bail!("unsupported abi.decode type-list element"),
        }
    }

    /// Maps a Solidity elementary type keyword (`uint<N>`, `int<N>`, `bytes`,
    /// `bytes<N>`, `bool`, `address`, `string`) to its MLIR type, parsing the
    /// width from the keyword's source text (`uint`/`int` default to 256 bits).
    pub(in crate::ast::contract::function::expression::call) fn resolve_abi_elementary_type(
        &self,
        elementary: &ElementaryType,
    ) -> anyhow::Result<Type<'context>> {
        let builder = &self.expression_emitter.state.builder;
        let parse_width = |text: &str, prefix: &str| -> anyhow::Result<u32> {
            match text.trim().strip_prefix(prefix) {
                Some("") => Ok(256),
                Some(digits) => digits
                    .parse::<u32>()
                    .map_err(|_| anyhow::anyhow!("invalid `{prefix}` width in `{text}`")),
                None => anyhow::bail!("`{text}` is not a `{prefix}` type"),
            }
        };
        let resolved = match elementary {
            ElementaryType::BoolKeyword(_) => builder.types.i1,
            ElementaryType::AddressType(_) => builder.types.sol_address,
            ElementaryType::StringKeyword(_) => {
                builder.types.string(solx_utils::DataLocation::Memory)
            }
            ElementaryType::UintKeyword(keyword) => {
                let bits = parse_width(keyword.unparse(), "uint")?;
                Type::from(IntegerType::unsigned(builder.context, bits))
            }
            ElementaryType::IntKeyword(keyword) => {
                let bits = parse_width(keyword.unparse(), "int")?;
                Type::from(IntegerType::signed(builder.context, bits))
            }
            ElementaryType::BytesKeyword(keyword) => {
                let text = keyword.unparse();
                if text.trim() == "bytes" {
                    builder.types.string(solx_utils::DataLocation::Memory)
                } else {
                    builder.types.fixed_bytes(parse_width(text, "bytes")?)
                }
            }
            ElementaryType::FixedKeyword(_) | ElementaryType::UfixedKeyword(_) => {
                anyhow::bail!("fixed-point types are not supported")
            }
        };
        Ok(resolved)
    }

    /// Determines the MLIR result types of `abi.decode(payload, (T1, T2, …))`.
    ///
    /// Slang types most decode calls correctly, but leaves an elementary
    /// type-name argument (`uint256`, `bytes`, …) untyped — the call's type is
    /// a tuple whose corresponding element is `Void`. Those positions are
    /// reconstructed from the type-list argument via
    /// [`Self::resolve_abi_type_expression`]; every other position keeps
    /// slang's type, so arrays, structs, enums, and user-defined value types
    /// continue to resolve through the binder.
    fn abi_decode_result_types(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
    ) -> anyhow::Result<Vec<Type<'context>>> {
        let builder = &self.expression_emitter.state.builder;
        let argument_types: Vec<Expression> = match arguments.iter().nth(1) {
            Some(Expression::TupleExpression(tuple)) => {
                tuple.items().iter().filter_map(|item| item.expression()).collect()
            }
            Some(other) => vec![other],
            None => Vec::new(),
        };
        let slang_types: Vec<SlangType> = match call.get_type() {
            Some(SlangType::Tuple(tuple)) => tuple.types(),
            Some(other) => vec![other],
            None => Vec::new(),
        };
        let count = slang_types.len().max(argument_types.len());
        (0..count)
            .map(|index| {
                // Prefer slang's type whenever it is meaningful.
                if let Some(slang_type) = slang_types.get(index)
                    && !matches!(slang_type, SlangType::Void(_))
                {
                    return Ok(TypeConversion::resolve_slang_type(slang_type, None, builder));
                }
                // Slang left this position untyped (`Void`). If the type-list
                // argument names an elementary type, reconstruct it; otherwise
                // fall back to slang's resolution (`Void` -> `ui256`), preserving
                // prior behaviour for 256-bit-word decodes such as user-defined
                // value types.
                if let Some(argument) = argument_types.get(index)
                    && let Ok(resolved) = self.resolve_abi_type_expression(argument)
                {
                    return Ok(resolved);
                }
                Ok(slang_types.get(index).map_or(builder.types.ui256, |slang_type| {
                    TypeConversion::resolve_slang_type(slang_type, None, builder)
                }))
            })
            .collect()
    }

    /// Emits `abi.decode(payload, (T1, T2, …))` as a `sol.decode` operation,
    /// yielding one value per requested type.
    pub(crate) fn emit_abi_decode(
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
        let result_types = self.abi_decode_result_types(call, arguments)?;
        let builder = &self.expression_emitter.state.builder;
        let operation = block.append_operation(
            DecodeOperation::builder(builder.context, builder.unknown_location)
                .addr(payload_value)
                .outs(&result_types)
                .build()
                .into(),
        );
        let values: Vec<Value<'context, 'block>> = (0..result_types.len())
            .map(|index| {
                operation
                    .result(index)
                    .expect("sol.decode yields one result per requested type")
                    .into()
            })
            .collect();
        Ok((values, block))
    }
}
