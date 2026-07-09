//!
//! Environment value producers: `block.*`, `tx.*`, `msg.*` globals, `gasleft()`, and `this`.
//!
//! Each is a nullary op whose only setter is its result type, so it reads as a `Value` constructor.
//!

use crate::Context;
use crate::Type;
use crate::Value;
use crate::ods::sol::BaseFeeOperation;
use crate::ods::sol::BlobBaseFeeOperation;
use crate::ods::sol::BlockNumberOperation;
use crate::ods::sol::CallValueOperation;
use crate::ods::sol::CallerOperation;
use crate::ods::sol::ChainIdOperation;
use crate::ods::sol::CoinbaseOperation;
use crate::ods::sol::DifficultyOperation;
use crate::ods::sol::GasLeftOperation;
use crate::ods::sol::GasLimitOperation;
use crate::ods::sol::GasPriceOperation;
use crate::ods::sol::GetCallDataOperation;
use crate::ods::sol::OriginOperation;
use crate::ods::sol::PrevRandaoOperation;
use crate::ods::sol::SigOperation;
use crate::ods::sol::ThisOperation;
use crate::ods::sol::TimestampOperation;

impl<'context> Value<'context> {
    /// Emits `sol.number`: `block.number`.
    pub fn block_number(context: &Context<'context>) -> Self {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::from(mlir_op!(context, BlockNumberOperation.val(field)))
    }

    /// Emits `sol.timestamp`: `block.timestamp`.
    pub fn block_timestamp(context: &Context<'context>) -> Self {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::from(mlir_op!(context, TimestampOperation.val(field)))
    }

    /// Emits `sol.coinbase`: `block.coinbase`.
    pub fn block_coinbase(context: &Context<'context>) -> Self {
        let account = Type::address(context.melior, false).into_mlir();
        Self::from(mlir_op!(context, CoinbaseOperation.addr(account)))
    }

    /// Emits `sol.difficulty`: `block.difficulty`.
    pub fn block_difficulty(context: &Context<'context>) -> Self {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::from(mlir_op!(context, DifficultyOperation.val(field)))
    }

    /// Emits `sol.prevrandao`: `block.prevrandao`.
    pub fn block_prev_randao(context: &Context<'context>) -> Self {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::from(mlir_op!(context, PrevRandaoOperation.val(field)))
    }

    /// Emits `sol.gaslimit`: `block.gaslimit`.
    pub fn block_gas_limit(context: &Context<'context>) -> Self {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::from(mlir_op!(context, GasLimitOperation.val(field)))
    }

    /// Emits `sol.basefee`: `block.basefee`.
    pub fn block_base_fee(context: &Context<'context>) -> Self {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::from(mlir_op!(context, BaseFeeOperation.val(field)))
    }

    /// Emits `sol.blobbasefee`: `block.blobbasefee`.
    pub fn block_blob_base_fee(context: &Context<'context>) -> Self {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::from(mlir_op!(context, BlobBaseFeeOperation.val(field)))
    }

    /// Emits `sol.chainid`: `block.chainid`.
    pub fn block_chain_id(context: &Context<'context>) -> Self {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::from(mlir_op!(context, ChainIdOperation.val(field)))
    }

    /// Emits `sol.origin`: `tx.origin`.
    pub fn tx_origin(context: &Context<'context>) -> Self {
        let account = Type::address(context.melior, false).into_mlir();
        Self::from(mlir_op!(context, OriginOperation.addr(account)))
    }

    /// Emits `sol.gasprice`: `tx.gasprice`.
    pub fn tx_gas_price(context: &Context<'context>) -> Self {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::from(mlir_op!(context, GasPriceOperation.val(field)))
    }

    /// Emits `sol.caller`: `msg.sender`.
    pub fn msg_sender(context: &Context<'context>) -> Self {
        let account = Type::address(context.melior, false).into_mlir();
        Self::from(mlir_op!(context, CallerOperation.addr(account)))
    }

    /// Emits `sol.callvalue`: `msg.value`.
    pub fn msg_value(context: &Context<'context>) -> Self {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::from(mlir_op!(context, CallValueOperation.val(field)))
    }

    /// Emits `sol.sig`: `msg.sig`, the four-byte call selector.
    pub fn msg_sig(context: &Context<'context>) -> Self {
        let selector = Type::fixed_bytes(context.melior, 4).into_mlir();
        Self::from(mlir_op!(context, SigOperation.val(selector)))
    }

    /// Emits `sol.get_call_data`: `msg.data`, the calldata byte slice.
    pub fn msg_data(context: &Context<'context>) -> Self {
        let calldata = Type::string(context.melior, solx_utils::DataLocation::CallData).into_mlir();
        Self::from(mlir_op!(context, GetCallDataOperation.addr(calldata)))
    }

    /// Emits `sol.gasleft`: `gasleft()`.
    pub fn gas_left(context: &Context<'context>) -> Self {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::from(mlir_op!(context, GasLeftOperation.val(field)))
    }

    /// Emits `sol.this`: the current contract's address, typed as `contract_type`.
    pub fn this(contract_type: Type<'context>, context: &Context<'context>) -> Self {
        Self::from(mlir_op!(context, ThisOperation.addr(contract_type)))
    }
}
