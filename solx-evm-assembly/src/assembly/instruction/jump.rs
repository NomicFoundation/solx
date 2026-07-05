//!
//! Translates the jump operations.
//!

use solx_codegen_evm::IEVMLAData;
use solx_codegen_evm::IEVMLAFunction;

///
/// Translates the unconditional jump.
///
pub fn unconditional<'ctx, C>(
    context: &mut C,
    destination: u64,
    stack_hash: u64,
    stack_height: usize,
) -> anyhow::Result<()>
where
    C: solx_codegen_evm::IEVMLAStack<'ctx>,
{
    let code_segment = context
        .code_segment()
        .ok_or_else(|| anyhow::anyhow!("Contract code segment is undefined"))?;
    let block_key = match code_segment {
        solx_utils::CodeSegment::Deploy if destination > u32::MAX as u64 => {
            solx_codegen_evm::BlockKey::new(
                solx_utils::CodeSegment::Runtime,
                destination - (1u64 << 32),
            )
        }
        code_segment => solx_codegen_evm::BlockKey::new(code_segment, destination),
    };

    context.evmla_stack_flush(stack_height)?;

    let block = context
        .current_function()
        .borrow()
        .find_block(&block_key, &stack_hash)?;
    context.build_unconditional_branch(block.inner())?;

    Ok(())
}

///
/// Translates the conditional jump.
///
pub fn conditional<'ctx, C>(
    context: &mut C,
    destination: u64,
    stack_hash: u64,
    stack_height: usize,
) -> anyhow::Result<()>
where
    C: solx_codegen_evm::IEVMLAStack<'ctx>,
{
    let code_segment = context
        .code_segment()
        .ok_or_else(|| anyhow::anyhow!("Contract code segment is undefined"))?;
    let block_key = match code_segment {
        solx_utils::CodeSegment::Deploy if destination > u32::MAX as u64 => {
            solx_codegen_evm::BlockKey::new(
                solx_utils::CodeSegment::Runtime,
                destination - (1u64 << 32),
            )
        }
        code_segment => solx_codegen_evm::BlockKey::new(code_segment, destination),
    };

    let condition = context.evmla_stack_read(stack_height)?;
    let condition = context.build_int_compare(
        inkwell::IntPredicate::NE,
        condition.into_int_value(),
        context.field_const(0),
        format!("conditional_{block_key}_condition_compared").as_str(),
    )?;

    context.evmla_stack_flush(stack_height)?;

    let then_block = context
        .current_function()
        .borrow()
        .find_block(&block_key, &stack_hash)?;
    let join_block =
        context.append_basic_block(format!("conditional_{block_key}_join_block").as_str());

    context.build_conditional_branch(condition, then_block.inner(), join_block)?;

    context.set_basic_block(join_block);
    context.evmla_mut().expect("Always exists").shadow_reset();

    Ok(())
}
