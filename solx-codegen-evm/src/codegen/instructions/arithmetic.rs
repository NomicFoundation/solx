//!
//! Translates the arithmetic operations.
//!

use inkwell::values::BasicValue;

use crate::codegen::context::Context;
use crate::context::IContext;

///
/// Translates the arithmetic addition.
///
/// There is not difference between the EVM and LLVM IR behaviors.
///
pub fn addition<'ctx>(
    context: &mut Context<'ctx>,
    operand_1: inkwell::values::IntValue<'ctx>,
    operand_2: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .builder()
        .build_int_add(operand_1, operand_2, "addition_result")?
        .as_basic_value_enum())
}

///
/// Translates the arithmetic subtraction.
///
/// There is not difference between the EVM and LLVM IR behaviors.
///
pub fn subtraction<'ctx>(
    context: &mut Context<'ctx>,
    operand_1: inkwell::values::IntValue<'ctx>,
    operand_2: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .builder()
        .build_int_sub(operand_1, operand_2, "subtraction_result")?
        .as_basic_value_enum())
}

///
/// Translates the arithmetic multiplication.
///
/// There is not difference between the EVM and LLVM IR behaviors.
///
pub fn multiplication<'ctx>(
    context: &mut Context<'ctx>,
    operand_1: inkwell::values::IntValue<'ctx>,
    operand_2: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    Ok(context
        .builder()
        .build_int_mul(operand_1, operand_2, "multiplication_result")?
        .as_basic_value_enum())
}

///
/// Translates the arithmetic division.
///
/// The only difference between the EVM and LLVM IR is that 0 must be returned in case of
/// division by zero.
///
pub fn division<'ctx>(
    context: &mut Context<'ctx>,
    operand_1: inkwell::values::IntValue<'ctx>,
    operand_2: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let zero_block = context.append_basic_block("division_zero");
    let non_zero_block = context.append_basic_block("division_non_zero");
    let join_block = context.append_basic_block("division_join");

    let result_pointer = context.build_alloca(context.field_type(), "division_result_pointer")?;
    let condition = context.builder().build_int_compare(
        inkwell::IntPredicate::EQ,
        operand_2,
        context.field_const(0),
        "division_is_divider_zero",
    )?;
    context.build_conditional_branch(condition, zero_block, non_zero_block)?;

    context.set_basic_block(non_zero_block);
    let result = context.builder().build_int_unsigned_div(
        operand_1,
        operand_2,
        "division_result_non_zero",
    )?;
    context.build_store(result_pointer, result)?;
    context.build_unconditional_branch(join_block)?;

    context.set_basic_block(zero_block);
    context.build_store(result_pointer, context.field_const(0))?;
    context.build_unconditional_branch(join_block)?;

    context.set_basic_block(join_block);
    let result = context.build_load(result_pointer, "division_result")?;
    Ok(result)
}

///
/// Translates the arithmetic remainder.
///
/// The only difference between the EVM and LLVM IR is that 0 must be returned in case of
/// division by zero.
///
pub fn remainder<'ctx>(
    context: &mut Context<'ctx>,
    operand_1: inkwell::values::IntValue<'ctx>,
    operand_2: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let zero_block = context.append_basic_block("remainder_zero");
    let non_zero_block = context.append_basic_block("remainder_non_zero");
    let join_block = context.append_basic_block("remainder_join");

    let result_pointer = context.build_alloca(context.field_type(), "remainder_result_pointer")?;
    let condition = context.builder().build_int_compare(
        inkwell::IntPredicate::EQ,
        operand_2,
        context.field_const(0),
        "remainder_is_modulo_zero",
    )?;
    context.build_conditional_branch(condition, zero_block, non_zero_block)?;

    context.set_basic_block(non_zero_block);
    let result = context.builder().build_int_unsigned_rem(
        operand_1,
        operand_2,
        "remainder_result_non_zero",
    )?;
    context.build_store(result_pointer, result)?;
    context.build_unconditional_branch(join_block)?;

    context.set_basic_block(zero_block);
    context.build_store(result_pointer, context.field_const(0))?;
    context.build_unconditional_branch(join_block)?;

    context.set_basic_block(join_block);
    let result = context.build_load(result_pointer, "remainder_result")?;
    Ok(result)
}

///
/// Translates the signed arithmetic division.
///
/// Two differences between the EVM and LLVM IR:
/// 1. In case of division by zero, 0 is returned.
/// 2. In case of overflow, the first argument is returned.
///
pub fn division_signed<'ctx>(
    context: &mut Context<'ctx>,
    operand_1: inkwell::values::IntValue<'ctx>,
    operand_2: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let zero_block = context.append_basic_block("division_signed_zero");
    let non_zero_block = context.append_basic_block("division_signed_non_zero");
    let overflow_block = context.append_basic_block("division_signed_overflow");
    let non_overflow_block = context.append_basic_block("division_signed_non_overflow");
    let join_block = context.append_basic_block("division_signed_join");

    let result_pointer =
        context.build_alloca(context.field_type(), "division_signed_result_pointer")?;
    let condition_is_divider_zero = context.builder().build_int_compare(
        inkwell::IntPredicate::EQ,
        operand_2,
        context.field_const(0),
        "division_signed_is_divider_zero",
    )?;
    context.build_conditional_branch(condition_is_divider_zero, zero_block, non_zero_block)?;

    context.set_basic_block(non_zero_block);
    let condition_is_divided_int_min = context.builder().build_int_compare(
        inkwell::IntPredicate::EQ,
        operand_1,
        context.field_const_str_hex(
            "8000000000000000000000000000000000000000000000000000000000000000",
        ),
        "division_signed_is_divided_int_min",
    )?;
    let condition_is_divider_minus_one = context.builder().build_int_compare(
        inkwell::IntPredicate::EQ,
        operand_2,
        context.field_type().const_all_ones(),
        "division_signed_is_divider_minus_one",
    )?;
    let condition_is_overflow = context.builder().build_and(
        condition_is_divided_int_min,
        condition_is_divider_minus_one,
        "division_signed_is_overflow",
    )?;
    context.build_conditional_branch(condition_is_overflow, overflow_block, non_overflow_block)?;

    context.set_basic_block(overflow_block);
    context.build_store(result_pointer, operand_1)?;
    context.build_unconditional_branch(join_block)?;

    context.set_basic_block(non_overflow_block);
    let result = context.builder().build_int_signed_div(
        operand_1,
        operand_2,
        "division_signed_result_non_zero",
    )?;
    context.build_store(result_pointer, result)?;
    context.build_unconditional_branch(join_block)?;

    context.set_basic_block(zero_block);
    context.build_store(result_pointer, context.field_const(0))?;
    context.build_unconditional_branch(join_block)?;

    context.set_basic_block(join_block);
    let result = context.build_load(result_pointer, "division_signed_result")?;
    Ok(result)
}

///
/// Translates the signed arithmetic remainder.
///
/// The only differences between the EVM and LLVM IR are that 0 must be returned in cases of
/// division by 0 or -1.
///
pub fn remainder_signed<'ctx>(
    context: &mut Context<'ctx>,
    operand_1: inkwell::values::IntValue<'ctx>,
    operand_2: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let zero_block = context.append_basic_block("remainder_signed_zero");
    let non_zero_block = context.append_basic_block("remainder_signed_non_zero");
    let non_minus_one_block = context.append_basic_block("remainder_signed_non_minus_one");
    let join_block = context.append_basic_block("remainder_signed_join");

    let result_pointer =
        context.build_alloca(context.field_type(), "remainder_signed_result_pointer")?;
    let condition = context.builder().build_int_compare(
        inkwell::IntPredicate::EQ,
        operand_2,
        context.field_const(0),
        "remainder_signed_is_modulo_zero",
    )?;
    context.build_conditional_branch(condition, zero_block, non_zero_block)?;

    context.set_basic_block(non_zero_block);
    let condition = context.builder().build_int_compare(
        inkwell::IntPredicate::EQ,
        operand_2,
        context.field_type().const_all_ones(),
        "remainder_signed_is_minus_one",
    )?;
    context.build_conditional_branch(condition, zero_block, non_minus_one_block)?;

    context.set_basic_block(non_minus_one_block);
    let result = context.builder().build_int_signed_rem(
        operand_1,
        operand_2,
        "remainder_signed_result_non_zero",
    )?;
    context.build_store(result_pointer, result)?;
    context.build_unconditional_branch(join_block)?;

    context.set_basic_block(zero_block);
    context.build_store(result_pointer, context.field_const(0))?;
    context.build_unconditional_branch(join_block)?;

    context.set_basic_block(join_block);
    let result = context.build_load(result_pointer, "remainder_signed_result")?;
    Ok(result)
}
