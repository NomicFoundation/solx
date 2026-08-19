//!
//! Translates the context getter instructions.
//!

use inkwell::values::BasicValue;

use crate::codegen::context::Context;
use crate::context::IContext;

///
/// Translates the `gas_limit` instruction.
///
pub fn gas_limit<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(context.intrinsics().gaslimit, &[], "gaslimit")?
        .expect("Always exists"))
}

///
/// Translates the `gas_price` instruction.
///
pub fn gas_price<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(context.intrinsics().gasprice, &[], "gasprice")?
        .expect("Always exists"))
}

///
/// Translates the `tx.origin` instruction.
///
pub fn origin<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(context.intrinsics().origin, &[], "origin")?
        .expect("Always exists"))
}

///
/// Translates the `chain_id` instruction.
///
pub fn chain_id<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(context.intrinsics().chainid, &[], "chainid")?
        .expect("Always exists"))
}

///
/// Translates the `block_number` instruction.
///
pub fn block_number<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(context.intrinsics().number, &[], "number")?
        .expect("Always exists"))
}

///
/// Translates the `block_timestamp` instruction.
///
pub fn block_timestamp<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(context.intrinsics().timestamp, &[], "timestamp")?
        .expect("Always exists"))
}

///
/// Translates the `block_hash` instruction.
///
pub fn block_hash<'ctx>(
    context: &mut Context<'ctx>,
    index: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(
            context.intrinsics().blockhash,
            &[index.as_basic_value_enum()],
            "blockhash",
        )?
        .expect("Always exists"))
}

///
/// Translates the `difficulty` instruction.
///
pub fn difficulty<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(context.intrinsics().difficulty, &[], "difficulty")?
        .expect("Always exists"))
}

///
/// Translates the `coinbase` instruction.
///
pub fn coinbase<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(context.intrinsics().coinbase, &[], "coinbase")?
        .expect("Always exists"))
}

///
/// Translates the `basefee` instruction.
///
pub fn basefee<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(context.intrinsics().basefee, &[], "basefee")?
        .expect("Always exists"))
}

///
/// Translates the `blobhash` instruction.
///
pub fn blob_hash<'ctx>(
    context: &mut Context<'ctx>,
    index: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(
            context.intrinsics().blobhash,
            &[index.as_basic_value_enum()],
            "blobhash",
        )?
        .expect("Always exists"))
}

///
/// Translates the `blobbasefee` instruction.
///
pub fn blob_basefee<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(context.intrinsics().blobbasefee, &[], "blobbasefee")?
        .expect("Always exists"))
}

///
/// Translates the `msize` instruction.
///
pub fn msize<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(context.intrinsics().msize, &[], "msize")?
        .expect("Always exists"))
}
