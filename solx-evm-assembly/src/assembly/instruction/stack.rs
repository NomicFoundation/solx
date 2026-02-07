//!
//! Translates the stack memory operations.
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
        .const_int_from_string(
            value.to_ascii_uppercase().as_str(),
            inkwell::types::StringRadix::Hexadecimal,
        )
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
/// Translates the stack memory duplicate.
///
pub fn dup<'ctx, C>(
    context: &mut C,
    offset: usize,
    height: usize,
    original: &mut Option<String>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    C: solx_codegen_evm::IContext<'ctx>,
{
    let element = context
        .evmla()
        .expect("Always exists")
        .get_element(height - offset - 1);
    let value = context.build_load(
        solx_codegen_evm::Pointer::new_stack_field(context, element.to_llvm().into_pointer_value()),
        format!("dup{offset}").as_str(),
    )?;

    element.original.clone_into(original);

    Ok(value)
}

///
/// Translates the stack memory swap.
///
pub fn swap<'ctx, C>(context: &mut C, offset: usize, height: usize) -> anyhow::Result<()>
where
    C: solx_codegen_evm::IContext<'ctx>,
{
    let top_element = context
        .evmla()
        .expect("Always exists")
        .get_element(height - 1)
        .to_owned();
    let top_pointer = solx_codegen_evm::Pointer::new_stack_field(
        context,
        top_element.to_llvm().into_pointer_value(),
    );
    let top_value = context.build_load(top_pointer, format!("swap{offset}_top_value").as_str())?;

    let swap_element = context
        .evmla()
        .expect("Always exists")
        .get_element(height - offset - 1)
        .to_owned();
    let swap_pointer = solx_codegen_evm::Pointer::new_stack_field(
        context,
        swap_element.to_llvm().into_pointer_value(),
    );
    let swap_value =
        context.build_load(swap_pointer, format!("swap{offset}_swap_value").as_str())?;

    if let Some(original) = swap_element.original {
        context
            .evmla_mut()
            .expect("Always exists")
            .set_original(height - 1, original.to_owned());
    }
    if let Some(original) = top_element.original {
        context
            .evmla_mut()
            .expect("Always exists")
            .set_original(height - offset - 1, original.to_owned());
    }

    context.build_store(top_pointer, swap_value)?;
    context.build_store(swap_pointer, top_value)?;

    Ok(())
}

///
/// Translates the stack memory pop.
///
pub fn pop<'ctx, C>(_context: &mut C) -> anyhow::Result<()>
where
    C: solx_codegen_evm::IContext<'ctx>,
{
    Ok(())
}
