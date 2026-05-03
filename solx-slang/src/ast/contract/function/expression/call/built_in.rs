//!
//! Solidity built-in function and EVM intrinsic lowering.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Operation;
use melior::ir::Value;
use slang_solidity::backend::built_ins::BuiltIn;
use slang_solidity::backend::ir::ast::Expression;
use slang_solidity::backend::ir::ast::MemberAccessExpression;
use slang_solidity::backend::ir::ast::PositionalArguments;
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
use solx_mlir::ods::sol::GasLeftOperation;
use solx_mlir::ods::sol::GasLimitOperation;
use solx_mlir::ods::sol::GasPriceOperation;
use solx_mlir::ods::sol::GetCallDataOperation;
use solx_mlir::ods::sol::OriginOperation;
use solx_mlir::ods::sol::PrevRandaoOperation;
use solx_mlir::ods::sol::SigOperation;
use solx_mlir::ods::sol::TimestampOperation;

use crate::ast::contract::function::expression::call::CallEmitter;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Tries to emit `callee(arguments)` as a Solidity built-in.
    ///
    /// Resolves the callee via slang's binder to a [`BuiltIn`] variant.
    /// On match, returns `Ok(Some((value, block)))`, where `value` is
    /// `Some(...)` for value-producing built-ins (e.g. `gasleft()`) and
    /// `None` for statement-style built-ins (e.g. `assert`, `require`).
    /// Returns `Ok(None)` if the callee is not a built-in and the caller
    /// should fall through to generic dispatch.
    ///
    /// # Errors
    ///
    /// Returns an error if the callee is a built-in but its arguments are
    /// malformed (e.g. non-string `require` message).
    pub fn try_emit_built_in_call(
        &self,
        callee: &Expression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        let Expression::Identifier(identifier) = callee else {
            return Ok(None);
        };
        let Some(built_in) = identifier.resolved_built_in() else {
            return Ok(None);
        };
        match built_in {
            BuiltIn::Assert if arguments.len() == 1 => {
                let condition = arguments.iter().next().expect("argument count verified");
                Ok(Some((None, self.emit_assert(&condition, block)?)))
            }
            BuiltIn::Require if matches!(arguments.len(), 1 | 2) => {
                let mut iter = arguments.iter();
                let condition = iter.next().expect("argument count verified");
                let message = match iter.next() {
                    Some(Expression::StringExpression(string_expression)) => {
                        let bytes = string_expression.value();
                        Some(String::from_utf8(bytes).expect("require message is valid UTF-8"))
                    }
                    Some(_) => anyhow::bail!("require message must be a string literal"),
                    None => None,
                };
                Ok(Some((
                    None,
                    self.emit_require(&condition, message.as_deref(), block)?,
                )))
            }
            BuiltIn::Gasleft if arguments.is_empty() => {
                let builder = &self.expression_emitter.state.builder;
                let value = block
                    .append_operation(
                        GasLeftOperation::builder(builder.context, builder.unknown_location)
                            .val(builder.types.ui256)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("gasleft always produces one result")
                    .into();
                Ok(Some((Some(value), block)))
            }
            _ => Ok(None),
        }
    }

    /// Emits a member access expression as an EVM intrinsic.
    ///
    /// Resolves the member via slang's binder to a specific `BuiltIn` variant
    /// and lowers it to the matching `sol.*` operation. Address-base intrinsics
    /// (`address.balance`, `address.codehash`, `address.code`) first evaluate
    /// the address operand and pass it as the operation's container address.
    ///
    /// # Errors
    ///
    /// Returns an error if the member access does not resolve to a recognized
    /// EVM intrinsic.
    pub fn emit_built_in_member_access(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let builder = &self.expression_emitter.state.builder;
        match access.member().resolved_built_in() {
            Some(BuiltIn::AddressBalance) => {
                self.emit_address_base_intrinsic(access, block, |address_value| {
                    BalanceOperation::builder(builder.context, builder.unknown_location)
                        .cont_addr(address_value)
                        .out(builder.types.ui256)
                        .build()
                        .into()
                })
            }
            Some(BuiltIn::AddressCodehash) => {
                self.emit_address_base_intrinsic(access, block, |address_value| {
                    CodeHashOperation::builder(builder.context, builder.unknown_location)
                        .cont_addr(address_value)
                        .out(builder.types.ui256)
                        .build()
                        .into()
                })
            }
            Some(BuiltIn::AddressCode) => {
                self.emit_address_base_intrinsic(access, block, |address_value| {
                    CodeOperation::builder(builder.context, builder.unknown_location)
                        .cont_addr(address_value)
                        .out(builder.types.sol_string_memory)
                        .build()
                        .into()
                })
            }
            resolved => {
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
                    _ => anyhow::bail!("unsupported member access: {}", access.member().name()),
                };
                let value = block
                    .append_operation(operation)
                    .result(0)
                    .expect("intrinsic always produces one result")
                    .into();
                Ok((value, block))
            }
        }
    }

    /// Emits an EVM intrinsic that reads from a per-address container, e.g.
    /// `address.balance` (`sol.balance`) or `address.codehash` (`sol.code_hash`).
    ///
    /// Evaluates the address operand, builds the operation via `build_op`, and
    /// extracts its single result.
    fn emit_address_base_intrinsic<F>(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
        build_op: F,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)>
    where
        F: FnOnce(Value<'context, 'block>) -> Operation<'context>,
    {
        let (address_value, block) = self
            .expression_emitter
            .emit_value(&access.operand(), block)?;
        let value = block
            .append_operation(build_op(address_value))
            .result(0)
            .expect("address-base intrinsic always produces one result")
            .into();
        Ok((value, block))
    }

    /// Emits an `assert(condition)` built-in via `sol.assert`.
    fn emit_assert(
        &self,
        condition: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        let (condition_value, block) = self.expression_emitter.emit_value(condition, block)?;
        let condition_boolean = self
            .expression_emitter
            .emit_is_nonzero(condition_value, &block);
        self.expression_emitter
            .state
            .builder
            .emit_sol_assert(condition_boolean, &block);
        Ok(block)
    }

    /// Emits a `require(condition)` or `require(condition, "message")` built-in
    /// via `sol.require`.
    fn emit_require(
        &self,
        condition: &Expression,
        message: Option<&str>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        let (condition_value, block) = self.expression_emitter.emit_value(condition, block)?;
        let condition_boolean = self
            .expression_emitter
            .emit_is_nonzero(condition_value, &block);
        self.expression_emitter
            .state
            .builder
            .emit_sol_require(condition_boolean, message, &block);
        Ok(block)
    }
}
