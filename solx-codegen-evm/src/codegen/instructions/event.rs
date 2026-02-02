//!
//! Translates a log or event call.
//!

use inkwell::values::BasicValue;

use crate::codegen::context::Context;
use crate::codegen::context::address_space::AddressSpace;
use crate::context::IContext;
use crate::context::pointer::Pointer;

///
/// Translates an event log call.
///
pub fn log<'ctx>(
    context: &mut Context<'ctx>,
    input_offset: inkwell::values::IntValue<'ctx>,
    input_length: inkwell::values::IntValue<'ctx>,
    topics: Vec<inkwell::values::IntValue<'ctx>>,
) -> anyhow::Result<()> {
    let input_offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        input_offset,
        format!("log{}_input_offset_pointer", topics.len()).as_str(),
    )?;

    match topics.len() {
        0 => context.build_call(
            context.intrinsics().log0,
            &[
                input_offset_pointer.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
            ],
            "log0",
        ),
        1 => context.build_call(
            context.intrinsics().log1,
            &[
                input_offset_pointer.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
                topics[0].as_basic_value_enum(),
            ],
            "log1",
        ),
        2 => context.build_call(
            context.intrinsics().log2,
            &[
                input_offset_pointer.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
                topics[0].as_basic_value_enum(),
                topics[1].as_basic_value_enum(),
            ],
            "log2",
        ),
        3 => context.build_call(
            context.intrinsics().log3,
            &[
                input_offset_pointer.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
                topics[0].as_basic_value_enum(),
                topics[1].as_basic_value_enum(),
                topics[2].as_basic_value_enum(),
            ],
            "log3",
        ),
        4 => context.build_call(
            context.intrinsics().log4,
            &[
                input_offset_pointer.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
                topics[0].as_basic_value_enum(),
                topics[1].as_basic_value_enum(),
                topics[2].as_basic_value_enum(),
                topics[3].as_basic_value_enum(),
            ],
            "log4",
        ),
        length => panic!("The number of topics must be from 0 to 4, found {length}"),
    }?;
    Ok(())
}
