//!
//! Translates the external code operations.
//!

use inkwell::values::BasicValue;

use crate::codegen::context::Context;
use crate::codegen::context::address_space::AddressSpace;
use crate::context::IContext;
use crate::context::pointer::Pointer;

///
/// Translates the `codesize` instruction.
///
pub fn size<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(context.intrinsics().codesize, &[], "codesize")?
        .expect("Always exists"))
}

///
/// Translates the `codecopy` instruction.
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
        "codecopy_destination_pointer",
    )?;

    let source = Pointer::new_with_offset(
        context,
        AddressSpace::Code,
        context.byte_type(),
        source_offset,
        "codecopy_source_pointer",
    )?;

    context.build_memcpy(
        context.intrinsics().memory_copy_from_code,
        destination,
        source,
        size,
        "codecopy_memcpy",
    )?;
    Ok(())
}

///
/// Translates the `dataoffset` instruction.
///
pub fn data_offset<'ctx>(
    context: &mut Context<'ctx>,
    object_name: &str,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let object_name = context
        .llvm()
        .metadata_node(&[context.llvm().metadata_string(object_name).into()]);

    Ok(context
        .build_call_metadata(
            context.intrinsics().dataoffset,
            &[object_name.into()],
            "dataoffset",
        )?
        .expect("Always exists"))
}

///
/// Translates the `datasize` instruction.
///
pub fn data_size<'ctx>(
    context: &mut Context<'ctx>,
    object_name: &str,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let object_name = context
        .llvm()
        .metadata_node(&[context.llvm().metadata_string(object_name).into()]);

    Ok(context
        .build_call_metadata(
            context.intrinsics().datasize,
            &[object_name.into()],
            "datasize",
        )?
        .expect("Always exists"))
}

///
/// Translates the `extcodesize` instruction.
///
pub fn ext_size<'ctx>(
    context: &mut Context<'ctx>,
    address: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(
            context.intrinsics().extcodesize,
            &[address.as_basic_value_enum()],
            "extcodesize",
        )?
        .expect("Always exists"))
}

///
/// Translates the `extcodecopy` instruction.
///
pub fn ext_copy<'ctx>(
    context: &mut Context<'ctx>,
    address: inkwell::values::IntValue<'ctx>,
    destination_offset: inkwell::values::IntValue<'ctx>,
    source_offset: inkwell::values::IntValue<'ctx>,
    size: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()> {
    let destination = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        destination_offset,
        "extcodecopy_destination_pointer",
    )?;

    let source = Pointer::new_with_offset(
        context,
        AddressSpace::Code,
        context.byte_type(),
        source_offset,
        "extcodecopy_source_pointer",
    )?;

    context.build_call(
        context.intrinsics().extcodecopy,
        &[
            address.as_basic_value_enum(),
            destination.as_basic_value_enum(),
            source.as_basic_value_enum(),
            size.as_basic_value_enum(),
        ],
        "extcodecopy",
    )?;
    Ok(())
}

///
/// Translates the `extcodehash` instruction.
///
pub fn ext_hash<'ctx>(
    context: &mut Context<'ctx>,
    address: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(
            context.intrinsics().extcodehash,
            &[address.as_basic_value_enum()],
            "extcodehash",
        )?
        .expect("Always exists"))
}
