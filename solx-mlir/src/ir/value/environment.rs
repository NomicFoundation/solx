//!
//! Environment value producers: `block.*`, `tx.*`, `msg.*` globals, `gasleft()`, and `this`.
//!
//! Each is a nullary op whose only setter is its result type, so it reads as a `Value` constructor.
//!

use melior::ir::BlockLike;

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

impl<'context, 'block> Value<'context, 'block> {
    /// Emits `sol.number`: `block.number`.
    pub fn block_number<B>(context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::new(mlir_op!(context, block, BlockNumberOperation.val(field)))
    }

    /// Emits `sol.timestamp`: `block.timestamp`.
    pub fn block_timestamp<B>(context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::new(mlir_op!(context, block, TimestampOperation.val(field)))
    }

    /// Emits `sol.coinbase`: `block.coinbase`.
    pub fn block_coinbase<B>(context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let account = Type::address(context.melior, false).into_mlir();
        Self::new(mlir_op!(context, block, CoinbaseOperation.addr(account)))
    }

    /// Emits `sol.difficulty`: `block.difficulty`.
    pub fn block_difficulty<B>(context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::new(mlir_op!(context, block, DifficultyOperation.val(field)))
    }

    /// Emits `sol.prevrandao`: `block.prevrandao`.
    pub fn block_prev_randao<B>(context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::new(mlir_op!(context, block, PrevRandaoOperation.val(field)))
    }

    /// Emits `sol.gaslimit`: `block.gaslimit`.
    pub fn block_gas_limit<B>(context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::new(mlir_op!(context, block, GasLimitOperation.val(field)))
    }

    /// Emits `sol.basefee`: `block.basefee`.
    pub fn block_base_fee<B>(context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::new(mlir_op!(context, block, BaseFeeOperation.val(field)))
    }

    /// Emits `sol.blobbasefee`: `block.blobbasefee`.
    pub fn block_blob_base_fee<B>(context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::new(mlir_op!(context, block, BlobBaseFeeOperation.val(field)))
    }

    /// Emits `sol.chainid`: `block.chainid`.
    pub fn block_chain_id<B>(context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::new(mlir_op!(context, block, ChainIdOperation.val(field)))
    }

    /// Emits `sol.origin`: `tx.origin`.
    pub fn tx_origin<B>(context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let account = Type::address(context.melior, false).into_mlir();
        Self::new(mlir_op!(context, block, OriginOperation.addr(account)))
    }

    /// Emits `sol.gasprice`: `tx.gasprice`.
    pub fn tx_gas_price<B>(context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::new(mlir_op!(context, block, GasPriceOperation.val(field)))
    }

    /// Emits `sol.caller`: `msg.sender`.
    pub fn msg_sender<B>(context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let account = Type::address(context.melior, false).into_mlir();
        Self::new(mlir_op!(context, block, CallerOperation.addr(account)))
    }

    /// Emits `sol.callvalue`: `msg.value`.
    pub fn msg_value<B>(context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::new(mlir_op!(context, block, CallValueOperation.val(field)))
    }

    /// Emits `sol.sig`: `msg.sig`, the four-byte call selector.
    pub fn msg_sig<B>(context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let selector = Type::fixed_bytes(context.melior, 4).into_mlir();
        Self::new(mlir_op!(context, block, SigOperation.val(selector)))
    }

    /// Emits `sol.get_call_data`: `msg.data`, the calldata byte slice.
    pub fn msg_data<B>(context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let calldata = Type::string(context.melior, solx_utils::DataLocation::CallData).into_mlir();
        Self::new(mlir_op!(
            context,
            block,
            GetCallDataOperation.addr(calldata)
        ))
    }

    /// Emits `sol.gasleft`: `gasleft()`.
    pub fn gas_left<B>(context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::new(mlir_op!(context, block, GasLeftOperation.val(field)))
    }

    /// Emits `sol.this`: the current contract's address, typed as `contract_type`.
    pub fn this<B>(contract_type: Type<'context>, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::new(mlir_op!(context, block, ThisOperation.addr(contract_type)))
    }
}
