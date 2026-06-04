//!
//! ABI encoding/decoding member built-ins: `abi.encode`, `abi.encodePacked`,
//! `abi.encodeWithSelector`, `abi.encodeWithSignature`, and `abi.decode`.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::Type as SlangType;

use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Lowers `abi.encode(args...)` to a standard `sol.encode` `bytes memory`.
    pub fn emit_abi_encode(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (values, block) = self.emit_argument_values(arguments, block)?;
        let result = self
            .expression_emitter
            .state
            .builder
            .emit_sol_encode(&values, None, false, &block);
        Ok((Some(result), block))
    }

    /// Lowers `abi.encodePacked(args...)` to a packed `sol.encode` (no
    /// per-element padding).
    pub fn emit_abi_encode_packed(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (values, block) = self.emit_argument_values(arguments, block)?;
        let result = self
            .expression_emitter
            .state
            .builder
            .emit_sol_encode(&values, None, true, &block);
        Ok((Some(result), block))
    }

    /// Lowers `abi.encodeWithSelector(selector, args...)`: the first argument is
    /// the explicit 4-byte selector, prepended to the encoded remaining
    /// arguments.
    pub fn emit_abi_encode_with_selector(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let mut iter = arguments.iter();
        let selector_expression = iter
            .next()
            .expect("abi.encodeWithSelector takes a selector argument");
        let (selector, mut block) = self
            .expression_emitter
            .emit_value(&selector_expression, block)?;
        let selector = {
            let builder = &self.expression_emitter.state.builder;
            builder.emit_sol_cast(selector, builder.types.fixed_bytes(4), &block)
        };
        let mut values = Vec::new();
        for argument in iter {
            let (value, next) = self.expression_emitter.emit_value(&argument, block)?;
            values.push(value);
            block = next;
        }
        let result = self.expression_emitter.state.builder.emit_sol_encode(
            &values,
            Some(selector),
            false,
            &block,
        );
        Ok((Some(result), block))
    }

    /// Lowers `abi.encodeWithSignature(signature, args...)`: the 4-byte selector
    /// is the high bytes of `keccak256(signature)`, prepended to the encoded
    /// remaining arguments.
    pub fn emit_abi_encode_with_signature(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let mut iter = arguments.iter();
        let signature_expression = iter
            .next()
            .expect("abi.encodeWithSignature takes a signature argument");
        let (selector, mut block) = self.emit_signature_selector(&signature_expression, block)?;
        let mut values = Vec::new();
        for argument in iter {
            let (value, next) = self.expression_emitter.emit_value(&argument, block)?;
            values.push(value);
            block = next;
        }
        let result = self.expression_emitter.state.builder.emit_sol_encode(
            &values,
            Some(selector),
            false,
            &block,
        );
        Ok((Some(result), block))
    }

    /// Computes the 4-byte selector of an `abi.encodeWithSignature` signature: a
    /// string literal is hashed at compile time, any other expression at runtime
    /// via `sol.keccak256`.
    fn emit_signature_selector(
        &self,
        signature: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        if let Expression::StringExpression(string) = signature {
            let builder = &self.expression_emitter.state.builder;
            let hash = solx_utils::Keccak256Hash::from_slice(&string.value());
            let selector_word = u32::from_be_bytes(
                hash.as_bytes()[..4]
                    .try_into()
                    .expect("keccak256 yields 32 bytes"),
            );
            let selector_type = Type::from(IntegerType::unsigned(builder.context, 32));
            let selector_int =
                builder.emit_sol_constant(i64::from(selector_word), selector_type, &block);
            let selector =
                builder.emit_sol_bytes_cast(selector_int, builder.types.fixed_bytes(4), &block);
            return Ok((selector, block));
        }

        let (signature_value, block) = self.expression_emitter.emit_value(signature, block)?;
        let builder = &self.expression_emitter.state.builder;
        let signature_value = TypeConversion::from_target_type(
            builder.types.sol_string_memory,
            builder,
        )
        .emit(signature_value, builder, &block);
        let hash = builder.emit_sol_keccak256(signature_value, &block);
        let selector = builder.emit_sol_bytes_cast(hash, builder.types.fixed_bytes(4), &block);
        Ok((selector, block))
    }

    /// Lowers `abi.decode(payload, (T, ...))` in value position to `sol.decode`,
    /// yielding the single decoded value.
    pub fn emit_abi_decode(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let payload_expression = arguments
            .iter()
            .next()
            .expect("abi.decode takes a payload argument");
        let (payload, block) = self
            .expression_emitter
            .emit_value(&payload_expression, block)?;
        let result_types = self.abi_decode_result_types(call);
        let mut values =
            self.expression_emitter
                .state
                .builder
                .emit_sol_decode(payload, &result_types, &block);
        let value = values
            .drain(..)
            .next()
            .expect("abi.decode in value position yields a single value");
        Ok((Some(value), block))
    }

    /// Resolves the MLIR result types of an `abi.decode` from slang's typing of
    /// the call. A `Void` position — slang leaves a bare elementary type-name
    /// argument untyped — defaults to the 256-bit word `ui256`.
    fn abi_decode_result_types(&self, call: &FunctionCallExpression) -> Vec<Type<'context>> {
        let builder = &self.expression_emitter.state.builder;
        let slang_types = match call.get_type() {
            Some(SlangType::Tuple(tuple)) => tuple.types(),
            Some(other) => vec![other],
            None => return vec![builder.types.ui256],
        };
        slang_types
            .iter()
            .map(|slang_type| match slang_type {
                SlangType::Void(_) => builder.types.ui256,
                other => TypeConversion::resolve_slang_type(other, None, builder),
            })
            .collect()
    }
}
