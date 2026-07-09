//!
//! Account value producers: queries and value transfers keyed on an address operand.
//!

use crate::Context;
use crate::Type;
use crate::Value;
use crate::ods::sol::BalanceOperation;
use crate::ods::sol::CodeHashOperation;
use crate::ods::sol::CodeOperation;
use crate::ods::sol::SendOperation;
use crate::ods::sol::TransferOperation;

impl<'context> Value<'context> {
    /// Emits `sol.balance`: the wei balance of `address`.
    pub fn balance(address: Self, context: &Context<'context>) -> Self {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::from(mlir_op!(
            context,
            BalanceOperation.cont_addr(address).out(field)
        ))
    }

    /// Emits `sol.code_hash`: the code hash of `address`.
    pub fn code_hash(address: Self, context: &Context<'context>) -> Self {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::from(mlir_op!(
            context,
            CodeHashOperation.cont_addr(address).out(field)
        ))
    }

    /// Emits `sol.code`: the deployed bytecode of `address` as `bytes memory`.
    pub fn code(address: Self, context: &Context<'context>) -> Self {
        let memory = Type::string(context.melior, solx_utils::DataLocation::Memory).into_mlir();
        Self::from(mlir_op!(
            context,
            CodeOperation.cont_addr(address).out(memory)
        ))
    }

    /// Emits `sol.send`: transfers `amount` wei to `address`, yielding the success `bool`.
    pub fn send(address: Self, amount: Self, context: &Context<'context>) -> Self {
        let status = Type::signless(context.melior, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir();
        Self::from(mlir_op!(
            context,
            SendOperation.addr(address).val(amount).status(status)
        ))
    }

    /// Emits `sol.transfer`: transfers `amount` wei to `address`, reverting on failure. Produces
    /// no value.
    pub fn transfer(address: Self, amount: Self, context: &Context<'context>) {
        mlir_op_void!(context, TransferOperation.addr(address).val(amount));
    }
}
