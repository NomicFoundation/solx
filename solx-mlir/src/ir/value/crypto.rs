//!
//! Cryptographic value producers: the `keccak256` opcode and the hash/recover precompiles.
//!
//! Each derives a digest (or a recovered address) from its input values, so it reads as a `Value`
//! constructor over the raw operands.
//!

use melior::ir::BlockLike;

use crate::Context;
use crate::Type;
use crate::Value;
use crate::ods::sol::EcrecoverOperation;
use crate::ods::sol::Keccak256Operation;
use crate::ods::sol::Ripemd160Operation;
use crate::ods::sol::Sha256Operation;

impl<'context, 'block> Value<'context, 'block> {
    /// Emits `sol.keccak256` over `data`, yielding a `bytes32` digest.
    pub fn keccak256<B>(data: Self, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let digest = Type::fixed_bytes(context.melior, 32).into_mlir();
        Self::new(mlir_op!(
            context,
            block,
            Keccak256Operation.addr(data).result(digest)
        ))
    }

    /// Emits `sol.ecrecover` (precompile `0x01`) recovering the signer address from `hash` and the
    /// `v`/`r`/`s` signature components.
    pub fn ecrecover<B>(
        hash: Self,
        v: Self,
        r: Self,
        s: Self,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let account = Type::address(context.melior, false).into_mlir();
        Self::new(mlir_op!(
            context,
            block,
            EcrecoverOperation.hash(hash).v(v).r(r).s(s).result(account)
        ))
    }

    /// Emits `sol.sha256` (precompile `0x02`) over `data`, yielding a `bytes32` digest.
    pub fn sha256<B>(data: Self, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let digest = Type::fixed_bytes(context.melior, 32).into_mlir();
        Self::new(mlir_op!(
            context,
            block,
            Sha256Operation.data(data).result(digest)
        ))
    }

    /// Emits `sol.ripemd160` (precompile `0x03`) over `data`, yielding a `bytes20` digest.
    pub fn ripemd160<B>(data: Self, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let digest = Type::fixed_bytes(context.melior, 20).into_mlir();
        Self::new(mlir_op!(
            context,
            block,
            Ripemd160Operation.data(data).result(digest)
        ))
    }
}
