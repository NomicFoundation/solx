//!
//! Translates the comparison operations.
//!

use inkwell::values::BasicValue;

use crate::codegen::context::Context;
use crate::context::IContext;

///
/// Translates the comparison operations.
///
/// There is not difference between the EVM and LLVM IR behaviors.
///
pub fn compare<'ctx>(
    context: &mut Context<'ctx>,
    operand_1: inkwell::values::IntValue<'ctx>,
    operand_2: inkwell::values::IntValue<'ctx>,
    operation: inkwell::IntPredicate,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let result = context.build_int_compare(operation, operand_1, operand_2, "comparison_result")?;
    let result = context.build_bit_cast_instruction(
        inkwell::builder::Builder::build_int_z_extend_or_bit_cast,
        result,
        context.field_type(),
        "comparison_result_extended",
    )?;
    Ok(result.as_basic_value_enum())
}
