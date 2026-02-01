//!
//! Translates the mathematics operation.
//!

use inkwell::values::BasicValue;

use crate::codegen::context::Context;
use crate::codegen::context::address_space::AddressSpace;
use crate::context::IContext;
use crate::context::pointer::Pointer;

///
/// Translates the `addmod` instruction.
///
pub fn add_mod<'ctx>(
    context: &mut Context<'ctx>,
    operand_1: inkwell::values::IntValue<'ctx>,
    operand_2: inkwell::values::IntValue<'ctx>,
    modulo: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(
            context.intrinsics().addmod,
            &[
                operand_1.as_basic_value_enum(),
                operand_2.as_basic_value_enum(),
                modulo.as_basic_value_enum(),
            ],
            "addmod",
        )?
        .expect("Always exists"))
}

///
/// Translates the `mulmod` instruction.
///
pub fn mul_mod<'ctx>(
    context: &mut Context<'ctx>,
    operand_1: inkwell::values::IntValue<'ctx>,
    operand_2: inkwell::values::IntValue<'ctx>,
    modulo: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(
            context.intrinsics().mulmod,
            &[
                operand_1.as_basic_value_enum(),
                operand_2.as_basic_value_enum(),
                modulo.as_basic_value_enum(),
            ],
            "mulmod",
        )?
        .expect("Always exists"))
}

///
/// Translates the `exp` instruction.
///
pub fn exponent<'ctx>(
    context: &mut Context<'ctx>,
    value: inkwell::values::IntValue<'ctx>,
    exponent: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(
            context.intrinsics().exp,
            &[value.as_basic_value_enum(), exponent.as_basic_value_enum()],
            "mulmod",
        )?
        .expect("Always exists"))
}

///
/// Translates the `signextend` instruction.
///
pub fn sign_extend<'ctx>(
    context: &mut Context<'ctx>,
    bytes: inkwell::values::IntValue<'ctx>,
    value: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .build_call(
            context.intrinsics().signextend,
            &[bytes.as_basic_value_enum(), value.as_basic_value_enum()],
            "signextend",
        )?
        .expect("Always exists"))
}

///
/// Translates the `keccak256` instruction.
///
pub fn keccak256<'ctx>(
    context: &mut Context<'ctx>,
    input_offset: inkwell::values::IntValue<'ctx>,
    input_length: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let input_offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        input_offset,
        "keccak256_input_offset_pointer",
    )?;

    Ok(context
        .build_call(
            context.intrinsics().sha3,
            &[
                input_offset_pointer.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
            ],
            "keccak256",
        )?
        .expect("Always exists"))
}
