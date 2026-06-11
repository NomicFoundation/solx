//!
//! EVM environment globals (`block`/`tx`/`msg`) and unary member intrinsics
//! (`address.balance`/`code`/`codehash`, `.length`, `address.send`/`transfer`).
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use solx_mlir::ods::sol::BalanceOperation;
use solx_mlir::ods::sol::BaseFeeOperation;
use solx_mlir::ods::sol::BlobBaseFeeOperation;
use solx_mlir::ods::sol::BlockNumberOperation;
use solx_mlir::ods::sol::CallValueOperation;
use solx_mlir::ods::sol::CallerOperation;
use solx_mlir::ods::sol::ChainIdOperation;
use solx_mlir::ods::sol::CodeHashOperation;
use solx_mlir::ods::sol::CodeOperation;
use solx_mlir::ods::sol::CoinbaseOperation;
use solx_mlir::ods::sol::DifficultyOperation;
use solx_mlir::ods::sol::GasLimitOperation;
use solx_mlir::ods::sol::GasPriceOperation;
use solx_mlir::ods::sol::GetCallDataOperation;
use solx_mlir::ods::sol::LengthOperation;
use solx_mlir::ods::sol::OriginOperation;
use solx_mlir::ods::sol::PrevRandaoOperation;
use solx_mlir::ods::sol::SendOperation;
use solx_mlir::ods::sol::SigOperation;
use solx_mlir::ods::sol::TimestampOperation;
use solx_mlir::ods::sol::TransferOperation;

use crate::ast::contract::function::expression::call::CallEmitter;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Emits `address.balance` as `sol.balance`.
    pub fn emit_address_balance(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let builder = &self.expression_emitter.state.builder;
        self.emit_unary_member_intrinsic(access, block, |address_value| {
            BalanceOperation::builder(builder.context, builder.unknown_location)
                .cont_addr(address_value)
                .out(builder.types.ui256)
                .build()
                .into()
        })
    }

    /// Emits `address.codehash` as `sol.code_hash`.
    pub fn emit_address_codehash(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let builder = &self.expression_emitter.state.builder;
        self.emit_unary_member_intrinsic(access, block, |address_value| {
            CodeHashOperation::builder(builder.context, builder.unknown_location)
                .cont_addr(address_value)
                .out(builder.types.ui256)
                .build()
                .into()
        })
    }

    /// Emits `address.code` as `sol.code`.
    pub fn emit_address_code(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let builder = &self.expression_emitter.state.builder;
        self.emit_unary_member_intrinsic(access, block, |address_value| {
            CodeOperation::builder(builder.context, builder.unknown_location)
                .cont_addr(address_value)
                .out(builder.types.sol_string_memory)
                .build()
                .into()
        })
    }

    /// Emits `arr.length` / `bytes.length` / `string.length` as `sol.length`.
    pub fn emit_member_length(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let builder = &self.expression_emitter.state.builder;
        self.emit_unary_member_intrinsic(access, block, |operand| {
            LengthOperation::builder(builder.context, builder.unknown_location)
                .inp(operand)
                .len(builder.types.ui256)
                .build()
                .into()
        })
    }

    /// Emits `address.send(value)` as `sol.send`, yielding the success flag.
    pub fn emit_address_send(
        &self,
        access: &MemberAccessExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let builder = &self.expression_emitter.state.builder;
        let (addr, block) = self
            .expression_emitter
            .emit_value(&access.operand(), block)?;
        let (values, block) = self.emit_argument_values(arguments, block)?;
        let value = block
            .append_operation(
                SendOperation::builder(builder.context, builder.unknown_location)
                    .addr(addr)
                    .val(values[0])
                    .status(builder.types.i1)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("send always produces one result")
            .into();
        Ok((Some(value), block))
    }

    /// Emits `address.transfer(value)` as `sol.transfer` (no result value).
    pub fn emit_address_transfer(
        &self,
        access: &MemberAccessExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let builder = &self.expression_emitter.state.builder;
        let (addr, block) = self
            .expression_emitter
            .emit_value(&access.operand(), block)?;
        let (values, block) = self.emit_argument_values(arguments, block)?;
        // `sol.transfer` takes a `ui256` amount; a narrow literal (`x.transfer(1)`
        // → ui8) must be widened first.
        let amount = builder.emit_sol_cast(values[0], builder.types.ui256, &block);
        block.append_operation(
            TransferOperation::builder(builder.context, builder.unknown_location)
                .addr(addr)
                .val(amount)
                .build()
                .into(),
        );
        Ok((None, block))
    }

    /// Emits a nullary EVM environment global (`tx.origin`, `msg.sender`,
    /// `block.timestamp`, …) as its matching `sol.*` operation.
    ///
    /// `resolved` is the member's resolved built-in; an unrecognized member
    /// is a loud error.
    pub fn emit_environment_global(
        &self,
        resolved: Option<BuiltIn>,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let builder = &self.expression_emitter.state.builder;
        let operation = match resolved {
            Some(BuiltIn::TxOrigin) => {
                OriginOperation::builder(builder.context, builder.unknown_location)
                    .addr(builder.types.sol_address)
                    .build()
                    .into()
            }
            Some(BuiltIn::TxGasPrice) => {
                GasPriceOperation::builder(builder.context, builder.unknown_location)
                    .val(builder.types.ui256)
                    .build()
                    .into()
            }
            Some(BuiltIn::MsgSender) => {
                CallerOperation::builder(builder.context, builder.unknown_location)
                    .addr(builder.types.sol_address)
                    .build()
                    .into()
            }
            Some(BuiltIn::MsgValue) => {
                CallValueOperation::builder(builder.context, builder.unknown_location)
                    .val(builder.types.ui256)
                    .build()
                    .into()
            }
            Some(BuiltIn::BlockTimestamp) => {
                TimestampOperation::builder(builder.context, builder.unknown_location)
                    .val(builder.types.ui256)
                    .build()
                    .into()
            }
            Some(BuiltIn::BlockNumber) => {
                BlockNumberOperation::builder(builder.context, builder.unknown_location)
                    .val(builder.types.ui256)
                    .build()
                    .into()
            }
            Some(BuiltIn::BlockCoinbase) => {
                CoinbaseOperation::builder(builder.context, builder.unknown_location)
                    .addr(builder.types.sol_address)
                    .build()
                    .into()
            }
            Some(BuiltIn::BlockChainid) => {
                ChainIdOperation::builder(builder.context, builder.unknown_location)
                    .val(builder.types.ui256)
                    .build()
                    .into()
            }
            Some(BuiltIn::BlockBasefee) => {
                BaseFeeOperation::builder(builder.context, builder.unknown_location)
                    .val(builder.types.ui256)
                    .build()
                    .into()
            }
            Some(BuiltIn::BlockGaslimit) => {
                GasLimitOperation::builder(builder.context, builder.unknown_location)
                    .val(builder.types.ui256)
                    .build()
                    .into()
            }
            Some(BuiltIn::BlockBlobbasefee) => {
                BlobBaseFeeOperation::builder(builder.context, builder.unknown_location)
                    .val(builder.types.ui256)
                    .build()
                    .into()
            }
            Some(BuiltIn::BlockDifficulty) => {
                DifficultyOperation::builder(builder.context, builder.unknown_location)
                    .val(builder.types.ui256)
                    .build()
                    .into()
            }
            Some(BuiltIn::BlockPrevrandao) => {
                PrevRandaoOperation::builder(builder.context, builder.unknown_location)
                    .val(builder.types.ui256)
                    .build()
                    .into()
            }
            Some(BuiltIn::MsgSig) => {
                SigOperation::builder(builder.context, builder.unknown_location)
                    .val(builder.types.fixed_bytes(4))
                    .build()
                    .into()
            }
            Some(BuiltIn::MsgData) => {
                GetCallDataOperation::builder(builder.context, builder.unknown_location)
                    .addr(builder.types.string(solx_utils::DataLocation::CallData))
                    .build()
                    .into()
            }
            // TODO: split this catch-all so non-built-in member accesses (struct fields, etc.) and unimplemented built-ins surface distinct errors.
            _ => unimplemented!("unsupported member access: {}", access.member().name()),
        };
        let value = block
            .append_operation(operation)
            .result(0)
            .expect("intrinsic always produces one result")
            .into();
        Ok((Some(value), block))
    }
}
