//!
//! The entry function.
//!

use crate::codegen::WriteLLVM;
use crate::codegen::attribute::Attribute;
use crate::codegen::context::Context;
use crate::context::IContext;

///
/// The entry function.
///
/// Is a special runtime function that is only used by the front-end generated code.
///
#[derive(Debug)]
pub struct Entry<B>
where
    B: WriteLLVM,
{
    /// The runtime code AST representation.
    inner: B,
}

impl<B> Entry<B>
where
    B: WriteLLVM,
{
    ///
    /// A shortcut constructor.
    ///
    pub fn new(inner: B) -> Self {
        Self { inner }
    }
}

impl<B> WriteLLVM for Entry<B>
where
    B: WriteLLVM,
{
    fn declare(&mut self, context: &mut Context) -> anyhow::Result<()> {
        let function_type = context.function_type::<inkwell::types::BasicTypeEnum>(vec![], 0);
        let function = context.add_function(
            crate::r#const::ENTRY_FUNCTION_NAME,
            None,
            function_type,
            0,
            Some(inkwell::module::Linkage::External),
        )?;
        function.borrow().declaration().value.add_attribute(
            inkwell::attributes::AttributeLoc::Function,
            context
                .llvm()
                .create_string_attribute(Attribute::EVMEntryFunction.to_string().as_str(), ""),
        );

        self.inner.declare(context)
    }

    fn into_llvm(self, context: &mut Context) -> anyhow::Result<()> {
        context.set_current_function(crate::r#const::ENTRY_FUNCTION_NAME)?;

        context.set_basic_block(context.current_function().borrow().entry_block());
        self.inner.into_llvm(context)?;
        if !context.is_basic_block_terminated() {
            context
                .build_unconditional_branch(context.current_function().borrow().return_block())?;
        }

        context.set_basic_block(context.current_function().borrow().return_block());
        if !context.is_basic_block_terminated() {
            crate::codegen::instructions::r#return::stop(context)?;
        }

        Ok(())
    }
}
