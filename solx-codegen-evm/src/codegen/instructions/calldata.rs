//!
//! Translates the calldata instructions.
//!

use crate::codegen::context::Context;
use crate::codegen::context::address_space::AddressSpace;
use crate::context::IContext;
use crate::context::pointer::Pointer;

///
/// Translates the calldata load.
///
pub fn load<'ctx>(
    context: &mut Context<'ctx>,
    offset: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Calldata,
        context.field_type(),
        offset,
        "calldataload_pointer",
    )?;
    let result = context.build_load(pointer, "calldata_load_result")?;
    Ok(result)
}

///
/// Translates the calldata size.
///
pub fn size<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(context.intrinsics().calldatasize, &[], "calldatasize")?
        .expect("Always exists"))
}

///
/// Translates the calldata copy.
///
pub fn copy<'ctx>(
    context: &mut Context<'ctx>,
    destination_offset: inkwell::values::IntValue<'ctx>,
    source_offset: inkwell::values::IntValue<'ctx>,
    size: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()> {
    let destination = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        destination_offset,
        "calldatacopy_destination_pointer",
    )?;

    let source = Pointer::new_with_offset(
        context,
        AddressSpace::Calldata,
        context.byte_type(),
        source_offset,
        "calldatacopy_source_pointer",
    )?;

    context.build_memcpy(
        context.intrinsics().memory_copy_from_calldata,
        destination,
        source,
        size,
        "calldatacopy_memcpy",
    )?;
    Ok(())
}
