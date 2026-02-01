//!
//! Translates the return data instructions.
//!

use crate::codegen::context::Context;
use crate::codegen::context::address_space::AddressSpace;
use crate::context::IContext;
use crate::context::pointer::Pointer;

///
/// Translates the return data size.
///
pub fn size<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(context.intrinsics().returndatasize, &[], "returndatasize")?
        .expect("Always exists"))
}

///
/// Translates the return data copy.
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
        "returndatacopy_destination_pointer",
    )?;

    let source = Pointer::new_with_offset(
        context,
        AddressSpace::ReturnData,
        context.byte_type(),
        source_offset,
        "returndatacopy_source_pointer",
    )?;

    context.build_memcpy(
        context.intrinsics().memory_copy_from_return_data,
        destination,
        source,
        size,
        "returndatacopy_memcpy",
    )?;
    Ok(())
}
