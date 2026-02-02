//!
//! Translates the heap memory operations.
//!

use inkwell::values::BasicValue;

use crate::codegen::context::Context;
use crate::codegen::context::address_space::AddressSpace;
use crate::context::IContext;
use crate::context::pointer::Pointer;

///
/// Translates the `mload` instruction.
///
/// Uses the main heap.
///
pub fn load<'ctx>(
    context: &mut Context<'ctx>,
    offset: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.field_type(),
        offset,
        "memory_load_pointer",
    )?;
    let result = context.build_load(pointer, "memory_load_result")?;
    Ok(result)
}

///
/// Translates the `mstore` instruction.
///
/// Uses the main heap.
///
pub fn store<'ctx>(
    context: &mut Context<'ctx>,
    offset: inkwell::values::IntValue<'ctx>,
    value: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()> {
    let pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.field_type(),
        offset,
        "memory_store_pointer",
    )?;
    context.build_store(pointer, value)?;
    Ok(())
}

///
/// Translates the `mstore8` instruction.
///
/// Uses the main heap.
///
pub fn store_byte<'ctx>(
    context: &mut Context<'ctx>,
    offset: inkwell::values::IntValue<'ctx>,
    value: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()> {
    let pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        offset,
        "store_byte_pointer",
    )?;

    context.build_call(
        context.intrinsics().mstore8,
        &[pointer.as_basic_value_enum(), value.as_basic_value_enum()],
        "mstore8",
    )?;
    Ok(())
}
