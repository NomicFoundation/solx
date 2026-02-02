//!
//! Translates the CODECOPY use cases.
//!

use inkwell::values::BasicValue;

use solx_codegen_evm::IContext;

///
/// Translates the contract hash copying.
///
pub fn dependency<'ctx>(
    context: &mut solx_codegen_evm::Context<'ctx>,
    offset: inkwell::values::IntValue<'ctx>,
    value: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()> {
    let offset = solx_codegen_evm::Context::build_binary_operator(
        context,
        inkwell::builder::Builder::build_int_add,
        offset,
        context.field_const((solx_utils::BYTE_LENGTH_X32 + solx_utils::BYTE_LENGTH_FIELD) as u64),
        "datacopy_dependency_offset",
    )?;

    solx_codegen_evm::memory::store(context, offset, value)?;

    Ok(())
}

///
/// Translates the static data copying.
///
pub fn static_data<'ctx>(
    context: &mut solx_codegen_evm::Context<'ctx>,
    destination: inkwell::values::IntValue<'ctx>,
    source: &str,
) -> anyhow::Result<()> {
    let source = hex::decode(source).expect("Always valid");
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
