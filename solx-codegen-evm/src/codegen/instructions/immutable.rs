//!
//! Translates the contract immutable operations.
//!

use crate::codegen::context::address_space::AddressSpace;
use crate::codegen::context::Context;
use crate::context::pointer::Pointer;
use crate::context::traits::solidity_data::ISolidityData;
use crate::context::IContext;

///
/// Translates the `loadimmutable` instruction.
///
pub fn load<'ctx>(
    context: &mut Context<'ctx>,
    id: &str,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let id = context
        .llvm()
        .metadata_node(&[context.llvm().metadata_string(id).into()]);

    Ok(context
        .build_call_metadata(
            context.intrinsics().loadimmutable,
            &[id.into()],
            "load_immutable",
        )?
        .expect("Always exists"))
}

///
/// Translates the `setimmutable` instruction.
///
pub fn store<'ctx>(
    context: &mut Context<'ctx>,
    id: &str,
    base_offset: inkwell::values::IntValue<'ctx>,
    value: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()> {
    match context.code_segment() {
        Some(solx_utils::CodeSegment::Deploy) => {}
        Some(solx_utils::CodeSegment::Runtime) => {
            anyhow::bail!("Setting immutables is only allowed in deploy code");
        }
        None => {
            anyhow::bail!("Code segment is undefined");
        }
    }

    let offsets = match context
        .solidity_mut()
        .expect("Always exists")
        .immutable_offsets(id)
    {
        Some(offsets) => offsets,
        None => return Ok(()),
    };
    for offset in offsets.into_iter() {
        let immutable_offset = Context::build_binary_operator(
            context,
            inkwell::builder::Builder::build_int_add,
            base_offset,
            context.field_const(offset),
            "setimmutable_offset",
        )?;
        let immutable_pointer = Pointer::new_with_offset(
            context,
            AddressSpace::Heap,
            context.byte_type(),
            immutable_offset,
            "setimmutable_pointer",
        )?;
        context.build_store(immutable_pointer, value)?;
    }

    Ok(())
}
