//!
//! External/public library calls — `L.f(args)` lowered as a delegatecall to
//! the linked library object, plus the revert-bubbling helper used to
//! re-raise the callee's revert data on failure.
//!

use super::*;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Emits an external/public library call `L.f(args)` as a delegatecall to
    /// the linked library: ABI-encode `(selector, args)`, `sol.lib_addr "L"` for
    /// the address, `sol.bare_delegate_call`, re-revert on failure, then decode
    /// the return value. Returns the (single) decoded result.
    pub fn emit_library_external_call(
        &self,
        library_name: &str,
        function: &slang_solidity_v2::ast::FunctionDefinition,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (parameter_types, return_types) =
            TypeConversion::resolve_function_types(function, &self.expression_emitter.state.builder);
        let selector = function
            .compute_selector()
            .ok_or_else(|| anyhow::anyhow!("library function '{library_name}' has no selector"))?;

        // Evaluate and coerce the arguments to the declared parameter types.
        let mut argument_values = Vec::with_capacity(arguments.iter().count());
        let mut current_block = block;
        for (index, argument) in arguments.iter().enumerate() {
            let (value, next) = match parameter_types.get(index) {
                Some(&parameter_type) => self.expression_emitter.emit_value_for_target(
                    &argument,
                    parameter_type,
                    current_block,
                )?,
                None => self.expression_emitter.emit_value(&argument, current_block)?,
            };
            let builder = &self.expression_emitter.state.builder;
            let value = match parameter_types.get(index) {
                Some(&parameter_type) => {
                    TypeConversion::from_target_type(parameter_type, builder).emit(value, builder, &next)
                }
                None => value,
            };
            argument_values.push(value);
            current_block = next;
        }

        let builder = &self.expression_emitter.state.builder;
        // Build the calldata: the 4-byte selector followed by the ABI-encoded
        // argument tuple.
        let selector_unsigned = builder.emit_sol_constant(
            i64::from(selector),
            Type::from(IntegerType::unsigned(builder.context, 32)),
            &current_block,
        );
        let selector_bytes =
            builder.emit_sol_cast(selector_unsigned, builder.types.fixed_bytes(4), &current_block);
        let calldata =
            self.emit_sol_encode(&argument_values, Some(selector_bytes), false, &current_block);

        let builder = &self.expression_emitter.state.builder;
        let address = builder.emit_sol_lib_addr(library_name, &current_block);
        let gas = current_block
            .append_operation(
                GasLeftOperation::builder(builder.context, builder.unknown_location)
                    .val(builder.types.ui256)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("gasleft produces one result")
            .into();
        let call = current_block.append_operation(
            BareDelegateCallOperation::builder(builder.context, builder.unknown_location)
                .addr(address)
                .gas(gas)
                .inp(calldata)
                .status(builder.types.i1)
                .ret_data(builder.types.sol_string_memory)
                .build()
                .into(),
        );
        let status = call.result(0).expect("delegatecall status").into();
        let return_data: Value<'context, 'block> =
            call.result(1).expect("delegatecall returndata").into();

        // Revert (bubbling the callee's revert data) when the call failed.
        let (then_block, else_block) = builder.emit_sol_if(status, &current_block);
        builder.emit_sol_yield(&then_block);
        Self::emit_bubble_revert(builder, &else_block);
        builder.emit_sol_yield(&else_block);

        if return_types.is_empty() {
            return Ok((None, current_block));
        }
        let decoded = current_block
            .append_operation(
                DecodeOperation::builder(builder.context, builder.unknown_location)
                    .addr(return_data)
                    .outs(&return_types)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.decode yields one result per requested type")
            .into();
        Ok((Some(decoded), current_block))
    }

    /// Emits a raw re-revert of the entire current returndata
    /// (`returndatacopy` + `revert`). `yul.revert` is not a terminator, so the
    /// caller appends a structural terminator after it.
    fn emit_bubble_revert(
        builder: &solx_mlir::Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) {
        let i256 = Type::from(IntegerType::new(builder.context, 256));
        let size = block
            .append_operation(
                solx_mlir::ods::yul::ReturnDataSizeOperation::builder(
                    builder.context,
                    builder.unknown_location,
                )
                .out(i256)
                .build()
                .into(),
            )
            .result(0)
            .expect("yul.returndatasize produces one result")
            .into();
        let zero_unsigned = builder.emit_sol_constant(0, builder.types.ui256, block);
        let zero = builder.emit_sol_cast(zero_unsigned, i256, block);
        block.append_operation(
            solx_mlir::ods::yul::ReturnDataCopyOperation::builder(
                builder.context,
                builder.unknown_location,
            )
            .dst(zero)
            .src(zero)
            .size(size)
            .build()
            .into(),
        );
        block.append_operation(
            solx_mlir::ods::yul::RevertOperation::builder(builder.context, builder.unknown_location)
                .addr(zero)
                .size(size)
                .build()
                .into(),
        );
    }
}
