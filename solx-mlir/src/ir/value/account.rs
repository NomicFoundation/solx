//!
//! Account value producers: queries and value transfers keyed on an address operand.
//!

use melior::ir::BlockLike;

use crate::Context;
use crate::Type;
use crate::Value;
use crate::ods::sol::BalanceOperation;
use crate::ods::sol::CodeHashOperation;
use crate::ods::sol::CodeOperation;
use crate::ods::sol::SendOperation;
use crate::ods::sol::TransferOperation;

impl<'context, 'block> Value<'context, 'block> {
    /// Emits `sol.balance`: the wei balance of `address`.
    pub fn balance<B>(address: Self, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::new(mlir_op!(
            context,
            block,
            BalanceOperation.cont_addr(address).out(field)
        ))
    }

    /// Emits `sol.code_hash`: the code hash of `address`.
    pub fn code_hash<B>(address: Self, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::new(mlir_op!(
            context,
            block,
            CodeHashOperation.cont_addr(address).out(field)
        ))
    }

    /// Emits `sol.code`: the deployed bytecode of `address` as `bytes memory`.
    pub fn code<B>(address: Self, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let memory = Type::string(context.melior, solx_utils::DataLocation::Memory).into_mlir();
        Self::new(mlir_op!(
            context,
            block,
            CodeOperation.cont_addr(address).out(memory)
        ))
    }

    /// Emits `sol.send`: transfers `amount` wei to `address`, yielding the success `bool`.
    pub fn send<B>(address: Self, amount: Self, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let status = Type::signless(context.melior, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir();
        Self::new(mlir_op!(
            context,
            block,
            SendOperation.addr(address).val(amount).status(status)
        ))
    }

    /// Emits `sol.transfer`: transfers `amount` wei to `address`, reverting on failure. Produces
    /// no value.
    pub fn transfer<B>(address: Self, amount: Self, context: &Context<'context>, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        mlir_op_void!(context, block, TransferOperation.addr(address).val(amount));
    }
}
