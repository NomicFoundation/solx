//!
//! Member access expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::MemberAccessExpression;

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

    /// Lowers a struct-field read `s.field`, returning `Ok(None)` when the base
    /// is not a struct so the caller falls back to built-in member access.
    fn emit_struct_field(
        &self,
        _access: &MemberAccessExpression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Value<'context, 'block>, BlockRef<'context, 'block>)>> {
        Ok(None)
    }

    /// Lowers a nullary environment global (`msg.*`, `tx.*`, `block.*`) to its
    /// `sol.*` intrinsic. The `msg` / `tx` / `block` operand is a magic global
    /// with no runtime value, so it is not evaluated.
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

    /// Lowers an operand-bearing member intrinsic. At this layer that is an
    /// address member (`address.balance` / `.codehash` / `.code`); the address
    /// operand is evaluated and passed to the matching `sol.*` intrinsic.
    /// `x.length` joins in the index-access domain.
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
            _ => unimplemented!("member access: unary member"),
        };
        Ok((value, block))
    }
}
