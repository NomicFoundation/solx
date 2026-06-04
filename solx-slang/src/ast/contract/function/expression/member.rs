//!
//! Member access expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
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
