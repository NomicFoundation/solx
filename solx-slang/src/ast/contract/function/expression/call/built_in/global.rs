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

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::contract::function::expression::ExpressionContext;

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Emits `address.balance` as `sol.balance`.
    pub fn emit_address_balance(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let builder = &self.state.builder;
        self.emit_unary_member_intrinsic(access, block, |address_value| {
            sol_op_build!(
                builder,
                BalanceOperation.cont_addr(address_value).out(
                    crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                        .into_mlir()
                )
            )
        })
    }

    /// Emits `address.codehash` as `sol.code_hash`.
    pub fn emit_address_codehash(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let builder = &self.state.builder;
        self.emit_unary_member_intrinsic(access, block, |address_value| {
            sol_op_build!(
                builder,
                CodeHashOperation.cont_addr(address_value).out(
                    crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                        .into_mlir()
                )
            )
        })
    }

    /// Emits `address.code` as `sol.code`.
    pub fn emit_address_code(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let builder = &self.state.builder;
        self.emit_unary_member_intrinsic(access, block, |address_value| {
            sol_op_build!(
                builder,
                CodeOperation
                    .cont_addr(address_value)
                    .out(builder.types.sol_string_memory)
            )
        })
    }

    /// Emits `arr.length` / `bytes.length` / `string.length` as `sol.length`.
    pub fn emit_member_length(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let builder = &self.state.builder;
        self.emit_unary_member_intrinsic(access, block, |operand| {
            sol_op_build!(
                builder,
                LengthOperation.inp(operand).len(
                    crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                        .into_mlir()
                )
            )
        })
    }

    /// Emits `address.send(value)` as `sol.send`, yielding the success flag.
    pub fn emit_address_send(
        &self,
        access: &MemberAccessExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let builder = &self.state.builder;
        let BlockAnd { value: addr, block } = access.operand().emit(self, block)?;
        let (values, block) = self.emit_argument_values(arguments, block)?;
        // `sol.send` takes a `ui256` amount; a narrow literal (`r.send(0)` → ui8)
        // must be widened first, like `address.transfer`.
        let amount = crate::ast::Value::from(values[0])
            .cast(
                crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                    .into_mlir(),
                builder,
                &block,
            )
            .into_mlir();
        let value = sol_op!(
            builder,
            block,
            SendOperation.addr(addr.into_mlir()).val(amount).status(
                crate::ast::Type::signless(builder.context, solx_utils::BIT_LENGTH_BOOLEAN)
                    .into_mlir()
            )
        );
        Ok((Some(value), block))
    }

    /// Emits `address.transfer(value)` as `sol.transfer` (no result value).
    pub fn emit_address_transfer(
        &self,
        access: &MemberAccessExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let builder = &self.state.builder;
        let BlockAnd { value: addr, block } = access.operand().emit(self, block)?;
        let (values, block) = self.emit_argument_values(arguments, block)?;
        // `sol.transfer` takes a `ui256` amount; a narrow literal (`x.transfer(1)`
        // → ui8) must be widened first.
        let amount = crate::ast::Value::from(values[0])
            .cast(
                crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                    .into_mlir(),
                builder,
                &block,
            )
            .into_mlir();
        sol_op_void!(
            builder,
            block,
            TransferOperation.addr(addr.into_mlir()).val(amount)
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
        let builder = &self.state.builder;
        let operation = match resolved {
            Some(BuiltIn::TxOrigin) => {
                sol_op_build!(builder, OriginOperation.addr(builder.types.sol_address))
            }
            Some(BuiltIn::TxGasPrice) => {
                sol_op_build!(
                    builder,
                    GasPriceOperation.val(
                        crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir()
                    )
                )
            }
            Some(BuiltIn::MsgSender) => {
                sol_op_build!(builder, CallerOperation.addr(builder.types.sol_address))
            }
            Some(BuiltIn::MsgValue) => {
                sol_op_build!(
                    builder,
                    CallValueOperation.val(
                        crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir()
                    )
                )
            }
            Some(BuiltIn::BlockTimestamp) => {
                sol_op_build!(
                    builder,
                    TimestampOperation.val(
                        crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir()
                    )
                )
            }
            Some(BuiltIn::BlockNumber) => {
                sol_op_build!(
                    builder,
                    BlockNumberOperation.val(
                        crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir()
                    )
                )
            }
            Some(BuiltIn::BlockCoinbase) => {
                sol_op_build!(builder, CoinbaseOperation.addr(builder.types.sol_address))
            }
            Some(BuiltIn::BlockChainid) => {
                sol_op_build!(
                    builder,
                    ChainIdOperation.val(
                        crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir()
                    )
                )
            }
            Some(BuiltIn::BlockBasefee) => {
                sol_op_build!(
                    builder,
                    BaseFeeOperation.val(
                        crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir()
                    )
                )
            }
            Some(BuiltIn::BlockGaslimit) => {
                sol_op_build!(
                    builder,
                    GasLimitOperation.val(
                        crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir()
                    )
                )
            }
            Some(BuiltIn::BlockBlobbasefee) => {
                sol_op_build!(
                    builder,
                    BlobBaseFeeOperation.val(
                        crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir()
                    )
                )
            }
            Some(BuiltIn::BlockDifficulty) => {
                sol_op_build!(
                    builder,
                    DifficultyOperation.val(
                        crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir()
                    )
                )
            }
            Some(BuiltIn::BlockPrevrandao) => {
                sol_op_build!(
                    builder,
                    PrevRandaoOperation.val(
                        crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir()
                    )
                )
            }
            Some(BuiltIn::MsgSig) => {
                sol_op_build!(builder, SigOperation.val(builder.types.fixed_bytes(4)))
            }
            Some(BuiltIn::MsgData) => {
                sol_op_build!(
                    builder,
                    GetCallDataOperation
                        .addr(builder.types.string(solx_utils::DataLocation::CallData))
                )
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
