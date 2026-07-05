//!
//! Translates the stack operations.
//!

use inkwell::values::BasicValue;

use solx_codegen_evm::IEVMLAData;

///
/// Translates the ordinar value push.
///
pub fn push<'ctx, C>(
    context: &mut C,
    value: String,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    C: solx_codegen_evm::IContext<'ctx>,
{
    let result = context
        .field_type()
        .const_int_from_string(value.as_str(), inkwell::types::StringRadix::Hexadecimal)
        .ok_or_else(|| anyhow::anyhow!("Invalid hexadecimal PUSH value: {value}"))?
        .as_basic_value_enum();
    Ok(result)
}

///
/// Translates the block tag label push.
///
pub fn push_tag<'ctx, C>(
    context: &mut C,
    value: String,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    C: solx_codegen_evm::IContext<'ctx>,
{
    let result = context
        .field_type()
        .const_int_from_string(value.as_str(), inkwell::types::StringRadix::Decimal)
        .ok_or_else(|| anyhow::anyhow!("Invalid decimal PUSH_Tag value: {value}"))?;
    Ok(result.as_basic_value_enum())
}

///
/// Duplicates a stack element on the shadow stack.
///
pub fn dup<'ctx, C>(context: &mut C, offset: usize, height: usize)
where
    C: solx_codegen_evm::IContext<'ctx>,
{
    context
        .evmla_mut()
        .expect("Always exists")
        .shadow_dup(height - 1, height - offset - 1);
}

///
/// Swaps two stack elements on the shadow stack.
///
pub fn swap<'ctx, C>(context: &mut C, offset: usize, height: usize)
where
    C: solx_codegen_evm::IContext<'ctx>,
{
    context
        .evmla_mut()
        .expect("Always exists")
        .shadow_swap(height - 1, height - offset - 1);
}
