//!
//! Member access expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::MemberAccessExpression;

use super::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers a member access `operand.member`.
    ///
    /// Handles the EVM environment globals (`msg.*`, `tx.*`, `block.*`) and the
    /// address-operand members (`address.balance` / `.codehash` / `.code`).
    /// Struct fields, namespace-qualified reads, enum variants, and selectors
    /// defer to later domains.
    pub fn emit_member_access(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        match access.member().resolve_to_built_in() {
            Some(
                built_in @ (BuiltIn::AddressBalance
                | BuiltIn::AddressCodehash
                | BuiltIn::AddressCode),
            ) => self.emit_address_member(built_in, access, block),
            Some(built_in) => Ok((
                self.emit_environment_global(built_in, access, &block),
                block,
            )),
            None => unimplemented!("member access lowering: {}", access.member().name()),
        }
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

    /// Lowers an address-operand member (`address.balance` / `.codehash` /
    /// `.code`): the address operand is evaluated and passed to the matching
    /// `sol.*` intrinsic.
    fn emit_address_member(
        &self,
        built_in: BuiltIn,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (address, block) = self.emit_value(&access.operand(), block)?;
        let builder = &self.state.builder;
        let value = match built_in {
            BuiltIn::AddressBalance => builder.emit_sol_balance(address, &block),
            BuiltIn::AddressCodehash => builder.emit_sol_code_hash(address, &block),
            BuiltIn::AddressCode => builder.emit_sol_code(address, &block),
            _ => unreachable!("emit_address_member only handles address members"),
        };
        Ok((value, block))
    }
}
