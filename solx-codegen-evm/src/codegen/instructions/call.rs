//!
//! Translates a contract call.
//!

use inkwell::values::BasicValue;

use crate::codegen::context::Context;
use crate::codegen::context::address_space::AddressSpace;
use crate::context::IContext;
use crate::context::pointer::Pointer;

///
/// Translates an external call.
///
#[allow(clippy::too_many_arguments)]
pub fn call<'ctx>(
    context: &mut Context<'ctx>,
    gas: inkwell::values::IntValue<'ctx>,
    address: inkwell::values::IntValue<'ctx>,
    value: inkwell::values::IntValue<'ctx>,
    input_offset: inkwell::values::IntValue<'ctx>,
    input_length: inkwell::values::IntValue<'ctx>,
    output_offset: inkwell::values::IntValue<'ctx>,
    output_length: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let input_offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        input_offset,
        "call_input_offset_pointer",
    )?;
    let output_offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        output_offset,
        "call_output_offset_pointer",
    )?;

    Ok(context
        .build_call(
            context.intrinsics().call,
            &[
                gas.as_basic_value_enum(),
                address.as_basic_value_enum(),
                value.as_basic_value_enum(),
                input_offset_pointer.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
                output_offset_pointer.as_basic_value_enum(),
                output_length.as_basic_value_enum(),
            ],
            "call",
        )?
        .expect("Always exists"))
}

///
/// Translates a static call.
///
#[allow(clippy::too_many_arguments)]
pub fn static_call<'ctx>(
    context: &mut Context<'ctx>,
    gas: inkwell::values::IntValue<'ctx>,
    address: inkwell::values::IntValue<'ctx>,
    input_offset: inkwell::values::IntValue<'ctx>,
    input_length: inkwell::values::IntValue<'ctx>,
    output_offset: inkwell::values::IntValue<'ctx>,
    output_length: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let input_offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        input_offset,
        "staticcall_input_offset_pointer",
    )?;
    let output_offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        output_offset,
        "staticcall_output_offset_pointer",
    )?;

    Ok(context
        .build_call(
            context.intrinsics().staticcall,
            &[
                gas.as_basic_value_enum(),
                address.as_basic_value_enum(),
                input_offset_pointer.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
                output_offset_pointer.as_basic_value_enum(),
                output_length.as_basic_value_enum(),
            ],
            "static_call",
        )?
        .expect("Always exists"))
}

///
/// Translates a delegate call.
///
#[allow(clippy::too_many_arguments)]
pub fn delegate_call<'ctx>(
    context: &mut Context<'ctx>,
    gas: inkwell::values::IntValue<'ctx>,
    address: inkwell::values::IntValue<'ctx>,
    input_offset: inkwell::values::IntValue<'ctx>,
    input_length: inkwell::values::IntValue<'ctx>,
    output_offset: inkwell::values::IntValue<'ctx>,
    output_length: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let input_offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        input_offset,
        "delegatecall_input_offset_pointer",
    )?;
    let output_offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        output_offset,
        "delegatecall_output_offset_pointer",
    )?;

    Ok(context
        .build_call(
            context.intrinsics().delegatecall,
            &[
                gas.as_basic_value_enum(),
                address.as_basic_value_enum(),
                input_offset_pointer.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
                output_offset_pointer.as_basic_value_enum(),
                output_length.as_basic_value_enum(),
            ],
            "delegate_call",
        )?
        .expect("Always exists"))
}

///
/// Translates the `linkersymbol` instruction.
///
pub fn linker_symbol<'ctx>(
    context: &mut Context<'ctx>,
    path: &str,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call_metadata(
            context.intrinsics().linkersymbol,
            &[context
                .llvm()
                .metadata_node(&[context.llvm().metadata_string(path).into()])
                .into()],
            format!("linker_symbol_{path}").as_str(),
        )?
        .expect("Always exists"))
}
