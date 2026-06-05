//!
//! Member access expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::r#type::IntegerType;
use num_bigint::BigInt;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::Type as SlangType;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers a member access `operand.member`.
    ///
    /// A struct-field access (`s.field`) is tried first; otherwise the member
    /// is an EVM built-in — the environment globals (`msg.*`, `tx.*`,
    /// `block.*`), or an operand-bearing member (`address.balance` /
    /// `.codehash` / `.code`, `x.length`). Namespace-qualified reads, enum
    /// variants, and selectors defer to later domains.
    pub fn emit_member_access(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        if let Some(result) = self.emit_struct_field(access, block)? {
            return Ok(result);
        }
        if let Some(result) = self.try_emit_type_introspection(access, block)? {
            return Ok(result);
        }
        if let Some(result) = self.try_emit_enum_variant(access, block)? {
            return Ok(result);
        }
        if let Some(result) = self.try_emit_selector(access, block)? {
            return Ok(result);
        }
        match access.member().resolve_to_built_in() {
            Some(
                built_in @ (BuiltIn::AddressBalance
                | BuiltIn::AddressCodehash
                | BuiltIn::AddressCode
                | BuiltIn::Length),
            ) => self.emit_unary_member(built_in, access, block),
            Some(built_in) => Ok((
                self.emit_environment_global(built_in, access, &block),
                block,
            )),
            None => unimplemented!("member access lowering: {}", access.member().name()),
        }
    }

    /// Lowers `MyEnum.VARIANT` — the variant's declaration index as a `ui256`
    /// constant bridged to `!sol.enum<max>` via `sol.enum_cast`. The operand may
    /// be a bare enum name (`MyEnum.VARIANT`) or a qualified path whose operand
    /// is itself a member access (`C.MyEnum.VARIANT`). Returns `Ok(None)` when
    /// the member is not an enum variant, so the caller falls through.
    fn try_emit_enum_variant(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Value<'context, 'block>, BlockRef<'context, 'block>)>> {
        if !matches!(
            access.member().resolve_to_definition(),
            Some(Definition::EnumMember(_))
        ) {
            return Ok(None);
        }
        let Some(Definition::Enum(enum_definition)) =
            Self::resolve_operand_definition(&access.operand())
        else {
            return Ok(None);
        };
        let member_name = access.member().name();
        let Some(index) = enum_definition
            .members()
            .iter()
            .position(|member| member.name() == member_name)
        else {
            return Ok(None);
        };
        let builder = &self.state.builder;
        let member_count = enum_definition.members().iter().count();
        let enum_type = builder.types.enumeration_for_member_count(member_count);
        let raw = builder.emit_sol_constant(index as i64, builder.types.ui256, &block);
        let value = builder.emit_sol_enum_cast(raw, enum_type, &block);
        Ok(Some((value, block)))
    }

    /// Resolves a member-access operand to its definition, handling a bare enum
    /// name (`E.A`) and a qualified path (`C.E.A`, whose operand is itself a
    /// member access).
    fn resolve_operand_definition(operand: &Expression) -> Option<Definition> {
        match operand {
            Expression::Identifier(identifier) => identifier.resolve_to_definition(),
            Expression::MemberAccessExpression(access) => access.member().resolve_to_definition(),
            _ => None,
        }
    }

    /// Lowers `f.selector` / `E.selector` / `this.x.selector` to a compile-time
    /// selector constant: a 4-byte `bytes4` for a function / error / public-
    /// getter selector, or the full 32-byte keccak topic hash (`bytes32`) for an
    /// event. The base resolves to a function, error, event, or public state
    /// variable. A member named `selector` that resolves to a user function (a
    /// `using`-attached function taken as a value) and a function-pointer VALUE's
    /// runtime `.selector` are both refused (`Ok(None)`), deferring to later
    /// domains.
    fn try_emit_selector(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Value<'context, 'block>, BlockRef<'context, 'block>)>> {
        if access.member().name() != "selector"
            || matches!(
                access.member().resolve_to_definition(),
                Some(Definition::Function(_))
            )
        {
            return Ok(None);
        }

        // Each arm yields `(selector value, fixedbytes width in bytes)`: a 4-byte
        // `bytes4` for functions / errors / getters, the 32-byte keccak topic
        // hash for events.
        let selector_constant: Option<(BigInt, u32)> =
            match Self::resolve_operand_definition(&access.operand()) {
                Some(Definition::Function(function)) => function
                    .compute_selector()
                    .map(|selector| (BigInt::from(selector), 4)),
                Some(Definition::Error(error)) => error
                    .compute_selector()
                    .map(|selector| (BigInt::from(selector), 4)),
                Some(Definition::StateVariable(state_variable)) => state_variable
                    .compute_selector()
                    .map(|selector| (BigInt::from(selector), 4)),
                Some(Definition::Event(event)) => {
                    event.compute_canonical_signature().map(|signature| {
                        let hash = solx_utils::Keccak256Hash::from_slice(signature.as_bytes());
                        let value = BigInt::from_bytes_be(num_bigint::Sign::Plus, hash.as_bytes());
                        (value, 32)
                    })
                }
                _ => None,
            };
        let Some((value, width_bytes)) = selector_constant else {
            return Ok(None);
        };

        // The selector is a compile-time constant, but a `.selector` taken off a
        // member of a value expression (`h().f.selector`) still evaluates that
        // receiver for its side effects.
        let block = self.eval_selector_receiver_side_effects(access, block)?;
        let builder = &self.state.builder;
        // `!sol.fixedbytes<N>` rejects a bare integer attribute, so emit the
        // value as an integer of the matching width and bridge via `sol.bytes_cast`.
        let integer_type = Type::from(IntegerType::unsigned(builder.context, width_bytes * 8));
        let integer = builder.emit_constant(&value, integer_type, &block);
        let value =
            builder.emit_sol_bytes_cast(integer, builder.types.fixed_bytes(width_bytes), &block);
        Ok(Some((value, block)))
    }

    /// Evaluates the receiver of a `.selector` taken off a value member
    /// (`h().f.selector`) for its side effects; a name / namespace / type
    /// receiver has none and is skipped.
    fn eval_selector_receiver_side_effects(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        let Expression::MemberAccessExpression(inner) = access.operand() else {
            return Ok(block);
        };
        let receiver = inner.operand();
        if is_namespace_or_type_operand(&receiver) {
            return Ok(block);
        }
        let (_discarded, block) = self.emit_value(&receiver, block)?;
        Ok(block)
    }

    /// Lowers a struct-field read `s.field` to `sol.gep` + `sol.load`.
    ///
    /// Returns `Ok(None)` when the base is not a struct, so the caller falls
    /// back to built-in member-access lowering.
    fn emit_struct_field(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Value<'context, 'block>, BlockRef<'context, 'block>)>> {
        let Some((address, element_type, block)) = self.emit_struct_field_address(access, block)?
        else {
            return Ok(None);
        };
        let value = self
            .state
            .builder
            .emit_sol_load(address, element_type, &block)?;
        Ok(Some((value, block)))
    }

    /// Emits the address of `s.field` together with the field's element type,
    /// without the trailing load. Shared by the value read and the assignment
    /// lvalue path. Returns `Ok(None)` when the base is not a struct.
    pub fn emit_struct_field_address(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<
        Option<(
            Value<'context, 'block>,
            Type<'context>,
            BlockRef<'context, 'block>,
        )>,
    > {
        let base = access.operand();
        let Some(SlangType::Struct(struct_type)) = base.get_type() else {
            return Ok(None);
        };
        let Definition::Struct(struct_definition) = struct_type.definition() else {
            unreachable!("a Slang struct type always references a struct definition");
        };
        let field_index = match access.member().resolve_to_definition() {
            Some(Definition::StructMember(field)) => struct_definition
                .members()
                .iter()
                .position(|member| member.node_id() == field.node_id()),
            _ => None,
        }
        .expect("the binder resolves a struct field access to a member of its struct");

        let (base_value, block) = self.emit_value(&base, block)?;
        let builder = &self.state.builder;
        let index = builder.emit_sol_constant(field_index as i64, builder.types.ui64, &block);
        let element_type =
            solx_mlir::TypeFactory::element_type(base_value.r#type(), field_index as u64);
        let address = builder.emit_sol_gep(base_value, index, element_type, &block);
        Ok(Some((address, element_type, block)))
    }

    /// Lowers a nullary environment global to its `sol.*` intrinsic. The
    /// `msg` / `tx` / `block` operand is a magic global with no runtime value,
    /// so it is not evaluated.
    fn emit_environment_global(
        &self,
        built_in: BuiltIn,
        access: &MemberAccessExpression,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let builder = &self.state.builder;
        match built_in {
            BuiltIn::MsgSender => builder.emit_sol_caller(block),
            BuiltIn::MsgValue => builder.emit_sol_call_value(block),
            BuiltIn::MsgSig => builder.emit_sol_sig(block),
            BuiltIn::MsgData => builder.emit_sol_call_data(block),
            BuiltIn::TxOrigin => builder.emit_sol_origin(block),
            BuiltIn::TxGasPrice => builder.emit_sol_gas_price(block),
            BuiltIn::BlockTimestamp => builder.emit_sol_timestamp(block),
            BuiltIn::BlockNumber => builder.emit_sol_block_number(block),
            BuiltIn::BlockCoinbase => builder.emit_sol_coinbase(block),
            BuiltIn::BlockChainid => builder.emit_sol_chain_id(block),
            BuiltIn::BlockBasefee => builder.emit_sol_base_fee(block),
            BuiltIn::BlockGaslimit => builder.emit_sol_gas_limit(block),
            BuiltIn::BlockBlobbasefee => builder.emit_sol_blob_base_fee(block),
            BuiltIn::BlockDifficulty => builder.emit_sol_difficulty(block),
            BuiltIn::BlockPrevrandao => builder.emit_sol_prev_randao(block),
            _ => unimplemented!("member access lowering: {}", access.member().name()),
        }
    }

    /// Lowers an operand-bearing member intrinsic — the address members
    /// (`address.balance` / `.codehash` / `.code`) and `x.length` — by
    /// evaluating the operand and passing it to the matching `sol.*` op.
    fn emit_unary_member(
        &self,
        built_in: BuiltIn,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (operand, block) = self.emit_value(&access.operand(), block)?;
        let builder = &self.state.builder;
        let value = match built_in {
            BuiltIn::AddressBalance => builder.emit_sol_balance(operand, &block),
            BuiltIn::AddressCodehash => builder.emit_sol_code_hash(operand, &block),
            BuiltIn::AddressCode => builder.emit_sol_code(operand, &block),
            BuiltIn::Length => builder.emit_sol_length(operand, &block),
            _ => unreachable!("emit_unary_member only handles operand-bearing members"),
        };
        Ok((value, block))
    }
}

/// Whether an expression names a namespace or type (a contract / interface /
/// library / import / enum / struct / user-defined value type) rather than a
/// runtime value, so a `.selector` receiver built on it has no side effects to
/// evaluate.
fn is_namespace_or_type_operand(expression: &Expression) -> bool {
    let definition = match expression {
        Expression::Identifier(identifier) => identifier.resolve_to_definition(),
        Expression::MemberAccessExpression(member) => member.member().resolve_to_definition(),
        _ => return false,
    };
    matches!(
        definition,
        Some(
            Definition::Contract(_)
                | Definition::Interface(_)
                | Definition::Library(_)
                | Definition::Import(_)
                | Definition::ImportedSymbol(_)
                | Definition::Enum(_)
                | Definition::Struct(_)
                | Definition::UserDefinedValueType(_)
        )
    )
}
