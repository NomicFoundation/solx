//!
//! Member-access *reference* built-ins — a member used as a value rather than
//! a call: bare `T.wrap`/`T.unwrap` and `addr.transfer`/`send`/`call`
//! references (no-op placeholders), `MyEnum.VARIANT`, `funcPtr.address`,
//! `f.selector` (and error/event/getter selectors), and external
//! function-pointer values (`this.f` / `obj.f`).
//!

use super::*;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// `T.wrap` / `T.unwrap` referenced without a call (a discarded
    /// `(MyInt).wrap;` statement). The call forms are handled in the call
    /// dispatch; a bare reference is a no-op, so yield a placeholder.
    pub(crate) fn try_emit_wrap_unwrap_reference(
        &self,
        access: &MemberAccessExpression,
        arguments: Option<&PositionalArguments>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        if arguments.is_none()
            && matches!(
                access.member().resolve_to_built_in(),
                Some(slang_solidity_v2::ast::BuiltIn::Wrap | slang_solidity_v2::ast::BuiltIn::Unwrap)
            )
        {
            let builder = &self.expression_emitter.state.builder;
            let placeholder = builder.emit_sol_constant(0, builder.types.ui256, &block);
            return Ok(Some((Some(placeholder), block)));
        }
        Ok(None)
    }

    /// `addr.transfer` / `addr.send` / `addr.call` (and `delegatecall` /
    /// `staticcall`) referenced WITHOUT a call — e.g. a discarded
    /// `payable(this).transfer;` statement — is a member reference, not the
    /// transfer/call action (which the call dispatch handles). Evaluate the
    /// operand for its side effects and yield a placeholder.
    pub(crate) fn try_emit_address_action_reference(
        &self,
        access: &MemberAccessExpression,
        arguments: Option<&PositionalArguments>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        if arguments.is_none()
            && matches!(
                access.member().resolve_to_built_in(),
                Some(
                    slang_solidity_v2::ast::BuiltIn::AddressTransfer
                        | slang_solidity_v2::ast::BuiltIn::AddressSend
                        | slang_solidity_v2::ast::BuiltIn::AddressCall
                        | slang_solidity_v2::ast::BuiltIn::AddressDelegatecall
                        | slang_solidity_v2::ast::BuiltIn::AddressStaticcall
                )
            )
        {
            let (_operand, block) =
                self.expression_emitter.emit_value(&access.operand(), block)?;
            let builder = &self.expression_emitter.state.builder;
            let placeholder = builder.emit_sol_constant(0, builder.types.ui256, &block);
            return Ok(Some((Some(placeholder), block)));
        }
        Ok(None)
    }

    /// `MyEnum.VARIANT` — emit the variant index as a ui256 constant and
    /// bridge to `!sol.enum<max>` via `sol.enum_cast`. The receiver may be a
    /// bare enum name (`MyEnum.VARIANT`) or a qualified path whose operand is
    /// itself a member access (`C.MyEnum.VARIANT`, `base.MyEnum.VARIANT`).
    pub(crate) fn try_emit_enum_variant(
        &self,
        access: &MemberAccessExpression,
        arguments: Option<&PositionalArguments>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        if arguments.is_none()
            && matches!(
                access.member().resolve_to_definition(),
                Some(slang_solidity_v2::ast::Definition::EnumMember(_))
            )
            && let Some(slang_solidity_v2::ast::Definition::Enum(enum_definition)) =
                resolve_member_access_operand(&access.operand())
        {
            let member_name = access.member().name();
            if let Some(index) = enum_definition
                .members()
                .iter()
                .position(|member| member.name() == member_name)
            {
                let builder = &self.expression_emitter.state.builder;
                let member_count = enum_definition.members().iter().count();
                let enum_type = builder.types.enumeration_for_member_count(member_count);
                let raw =
                    builder.emit_sol_constant(index as i64, builder.types.ui256, &block);
                let value = builder.emit_sol_enum_cast(raw, enum_type, &block);
                return Ok(Some((Some(value), block)));
            }
        }
        Ok(None)
    }

    /// `funcPtr.address` — the address component of an external function-pointer
    /// value (`C(addr).f.address`), pulled out of the `!sol.ext_func_ref` at
    /// runtime via `sol.ext_func_addr` (mirrors the `.selector` runtime arm).
    pub(crate) fn try_emit_function_pointer_address(
        &self,
        access: &MemberAccessExpression,
        arguments: Option<&PositionalArguments>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        if arguments.is_none()
            && access.member().name() == "address"
            && let Some(SlangType::Function(_)) = access.operand().get_type()
        {
            let (operand_value, block) =
                self.expression_emitter.emit_value(&access.operand(), block)?;
            if solx_mlir::TypeFactory::is_sol_ext_function_ref(operand_value.r#type()) {
                let builder = &self.expression_emitter.state.builder;
                let address = block
                    .append_operation(
                        ExtFuncAddrOperation::builder(builder.context, builder.unknown_location)
                            .func(operand_value)
                            .result(builder.types.sol_address)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("sol.ext_func_addr always produces one result")
                    .into();
                return Ok(Some((Some(address), block)));
            }
        }
        Ok(None)
    }

    /// `f.selector` / `E.selector` / `this.x.selector` — compile-time
    /// selector constant, with the base resolving to a function, error,
    /// event, or public state variable. A user library function may also be
    /// named `selector` (attached via `using`); that is always a call, so
    /// the `arguments.is_none()` guard excludes it, and we additionally
    /// refuse a `selector` member that resolves to a user function for the
    /// contrived case of taking such a bound function as a value. Function,
    /// error, and getter selectors are 4-byte (`bytes4`); event selectors
    /// are the full 32-byte keccak topic hash (`bytes32`).
    pub(crate) fn try_emit_selector(
        &self,
        access: &MemberAccessExpression,
        arguments: Option<&PositionalArguments>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        if arguments.is_none()
            && access.member().name() == "selector"
            && !matches!(
                access.member().resolve_to_definition(),
                Some(Definition::Function(_))
            )
        {
            let operand_definition = match access.operand() {
                Expression::Identifier(identifier) => identifier.resolve_to_definition(),
                Expression::MemberAccessExpression(inner) => {
                    inner.member().resolve_to_definition()
                }
                _ => None,
            };
            let builder = &self.expression_emitter.state.builder;
            // Each arm yields (selector value, fixedbytes width in bytes): a
            // 4-byte `bytes4` for functions / errors / getters, the full
            // 32-byte keccak topic hash (`bytes32`) for events.
            let selector_constant: Option<(num_bigint::BigInt, u32)> = match &operand_definition {
                Some(Definition::Function(function)) => function
                    .compute_selector()
                    .map(|selector| (num_bigint::BigInt::from(selector), 4)),
                Some(Definition::Error(error)) => error
                    .compute_selector()
                    .map(|selector| (num_bigint::BigInt::from(selector), 4)),
                Some(Definition::StateVariable(state_variable)) => state_variable
                    .compute_selector()
                    .map(|selector| (num_bigint::BigInt::from(selector), 4)),
                Some(Definition::Event(event)) => {
                    event.compute_canonical_signature().map(|signature| {
                        let hash = solx_utils::Keccak256Hash::from_slice(signature.as_bytes());
                        let value =
                            num_bigint::BigInt::from_bytes_be(num_bigint::Sign::Plus, hash.as_bytes());
                        (value, 32)
                    })
                }
                _ => None,
            };
            if let Some((value, width_bytes)) = selector_constant {
                // `!sol.fixedbytes<N>` rejects a bare integer attribute, so emit
                // the value as an integer constant of the matching width and
                // bridge to fixedbytes via `sol.bytes_cast`.
                let integer_type =
                    Type::from(IntegerType::unsigned(builder.context, width_bytes * 8));
                let integer = builder.emit_constant(&value, integer_type, &block);
                let value =
                    builder.emit_sol_bytes_cast(integer, builder.types.fixed_bytes(width_bytes), &block);
                return Ok(Some((Some(value), block)));
            }
            // Fall back to a runtime selector extraction when the operand is an
            // external function-pointer VALUE rather than a statically-known
            // function (`fun.selector`, `(cond ? a : b).selector`). The static
            // arms above already handle named functions / errors / events /
            // getters; here `sol.ext_func_selector` pulls the 4-byte selector
            // out of the `!sol.ext_func_ref` value at runtime.
            if let Some(SlangType::Function(_)) = access.operand().get_type() {
                let (operand_value, block) = self
                    .expression_emitter
                    .emit_value(&access.operand(), block)?;
                if solx_mlir::TypeFactory::is_sol_ext_function_ref(operand_value.r#type()) {
                    let builder = &self.expression_emitter.state.builder;
                    let selector = block
                        .append_operation(
                            ExtFuncSelectorOperation::builder(
                                builder.context,
                                builder.unknown_location,
                            )
                            .func(operand_value)
                            .result(builder.types.fixed_bytes(4))
                            .build()
                            .into(),
                        )
                        .result(0)
                        .expect("sol.ext_func_selector always produces one result")
                        .into();
                    return Ok(Some((Some(selector), block)));
                }
            }
        }
        Ok(None)
    }

    /// `this.f` / `obj.f` used as a value (no call) is an external
    /// function pointer: `sol.ext_func_constant(addr, selector)`.
    pub(crate) fn try_emit_external_function_pointer(
        &self,
        access: &MemberAccessExpression,
        arguments: Option<&PositionalArguments>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        if arguments.is_none()
            && let Some(slang_solidity_v2::ast::Definition::Function(function_definition)) =
                access.member().resolve_to_definition()
            && let Some(selector) = function_definition.compute_selector()
        {
            let (parameter_types, return_types) = TypeConversion::resolve_function_types(
                &function_definition,
                &self.expression_emitter.state.builder,
            );
            let (receiver_value, current_block) = self
                .expression_emitter
                .emit_value(&access.operand(), block)?;
            let builder = &self.expression_emitter.state.builder;
            let address = builder.emit_sol_address_cast(
                receiver_value,
                builder.types.sol_address,
                &current_block,
            );
            let ext_ref_type = builder.types.ext_func_ref(&parameter_types, &return_types);
            let value =
                builder.emit_sol_ext_func_constant(address, selector, ext_ref_type, &current_block);
            return Ok(Some((Some(value), current_block)));
        }
        Ok(None)
    }
}
