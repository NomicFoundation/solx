//!
//! Translates the CODECOPY use cases.
//!

use inkwell::values::BasicValue;

use solx_codegen_evm::IContext;

///
/// Translates the static data copying.
///
pub fn static_data<'ctx>(
    context: &mut solx_codegen_evm::Context<'ctx>,
    destination: inkwell::values::IntValue<'ctx>,
    source: &str,
) -> anyhow::Result<()> {
    let source = hex::decode(source)
        .map_err(|error| anyhow::anyhow!("Invalid CODECOPY hex data: {error}"))?;
    let source_type = context.array_type(context.byte_type(), source.len());
    let source_global = context.module().add_global(
        source_type,
        Some(solx_codegen_evm::AddressSpace::Code.into()),
        "codecopy_bytes_global",
    );
    source_global.set_initializer(
        &context
            .llvm()
            .const_string(source.as_slice(), false)
            .as_basic_value_enum(),
    );
    source_global.set_constant(true);
    source_global.set_linkage(inkwell::module::Linkage::Private);
    let source_pointer = solx_codegen_evm::Pointer::new(
        source_type,
        solx_codegen_evm::AddressSpace::Code,
        source_global.as_pointer_value(),
    );

    let destination_pointer = solx_codegen_evm::Pointer::new_with_offset(
        context,
        solx_codegen_evm::AddressSpace::Heap,
        context.field_type(),
        destination,
        "codecopy_bytes_destination_pointer",
    )?;

    context.build_memcpy(
        context.intrinsics().memory_copy_from_code,
        destination_pointer,
        source_pointer,
        context.field_const(source.len() as u64),
        "codecopy_memcpy",
    )?;
    Ok(())
}
