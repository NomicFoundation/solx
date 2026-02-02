//!
//! Translates the contract storage operations.
//!

use crate::codegen::context::Context;
use crate::codegen::context::address_space::AddressSpace;
use crate::context::IContext;
use crate::context::pointer::Pointer;

///
/// Translates the contract storage load.
///
pub fn load<'ctx>(
    context: &mut Context<'ctx>,
    position: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let position_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Storage,
        context.field_type(),
        position,
        "storage_load_position_pointer",
    )?;
    let value = context.build_load(position_pointer, "storage_load_value")?;
    Ok(value)
}

///
/// Translates the contract storage store.
///
pub fn store<'ctx>(
    context: &mut Context<'ctx>,
    position: inkwell::values::IntValue<'ctx>,
    value: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()> {
    let position_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Storage,
        context.field_type(),
        position,
        "storage_store_position_pointer",
    )?;
    context.build_store(position_pointer, value)?;
    Ok(())
}

///
/// Translates the transient storage load.
///
pub fn transient_load<'ctx>(
    context: &mut Context<'ctx>,
    position: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let position_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::TransientStorage,
        context.field_type(),
        position,
        "transient_storage_load_position_pointer",
    )?;
    let value = context.build_load(position_pointer, "transient_storage_load_value")?;
    Ok(value)
}

///
/// Translates the transient storage store.
///
pub fn transient_store<'ctx>(
    context: &mut Context<'ctx>,
    position: inkwell::values::IntValue<'ctx>,
    value: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()> {
    let position_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::TransientStorage,
        context.field_type(),
        position,
        "transient_storage_store_position_pointer",
    )?;
    context.build_store(position_pointer, value)?;
    Ok(())
}
