//!
//! The function definition statement.
//!

use inkwell::types::BasicType;
use solx_codegen_evm::IContext;

use crate::declare_wrapper;
use crate::yul::parser::dialect::era::EraDialect;
use crate::yul::parser::wrapper::Wrap;

declare_wrapper!(
    solx_yul::yul::parser::statement::function_definition::FunctionDefinition<EraDialect>,
    FunctionDefinition
);

impl solx_codegen_evm::WriteLLVM for FunctionDefinition {
    fn declare(&mut self, context: &mut solx_codegen_evm::Context) -> anyhow::Result<()> {
        let argument_types: Vec<_> = self
            .0
            .arguments
            .iter()
            .map(|argument| {
                let yul_type = argument.r#type.to_owned().unwrap_or_default();
                yul_type.wrap().into_llvm(context).as_basic_type_enum()
            })
            .collect();

        let function_type = context.function_type(argument_types, self.0.result.len());

        context.add_function(
            self.0.identifier.as_str(),
            function_type,
            self.0.result.len(),
            Some(inkwell::module::Linkage::Private),
        )?;

        Ok(())
    }

    fn into_llvm(mut self, context: &mut solx_codegen_evm::Context) -> anyhow::Result<()> {
        context.set_current_function(self.0.identifier.as_str())?;
        let r#return = context.current_function().borrow().r#return();

        context.set_basic_block(context.current_function().borrow().entry_block());
        match r#return {
            solx_codegen_evm::FunctionReturn::None => {}
            solx_codegen_evm::FunctionReturn::Primitive { pointer } => {
                let identifier = self.0.result.pop().expect("Always exists");
                let r#type = identifier.r#type.unwrap_or_default();
                context.build_store(pointer, r#type.wrap().into_llvm(context).const_zero())?;
                context
                    .current_function()
                    .borrow_mut()
                    .insert_stack_pointer(identifier.inner, pointer);
            }
            solx_codegen_evm::FunctionReturn::Compound { pointer, .. } => {
                for (index, identifier) in self.0.result.into_iter().enumerate() {
                    let r#type = identifier
                        .r#type
                        .unwrap_or_default()
                        .wrap()
                        .into_llvm(context);
                    let pointer = context.build_gep(
                        pointer,
                        &[
                            context.field_const(0),
                            context
                                .integer_type(solx_utils::BIT_LENGTH_X32)
                                .const_int(index as u64, false),
                        ],
                        context.field_type(),
                        format!("return_{index}_gep_pointer").as_str(),
                    )?;
                    context.build_store(pointer, r#type.const_zero())?;
                    context
                        .current_function()
                        .borrow_mut()
                        .insert_stack_pointer(identifier.inner.clone(), pointer);
                }
            }
        };

        let argument_types: Vec<_> = self
            .0
            .arguments
            .iter()
            .map(|argument| {
                let yul_type = argument.r#type.to_owned().unwrap_or_default();
                yul_type.wrap().into_llvm(context)
            })
            .collect();
        for (index, argument) in self.0.arguments.iter().enumerate() {
            let pointer = context.build_alloca(argument_types[index], argument.inner.as_str())?;
            context
                .current_function()
                .borrow_mut()
                .insert_stack_pointer(argument.inner.clone(), pointer);
            context.build_store(
                pointer,
                context.current_function().borrow().get_nth_param(index),
            )?;
        }

        self.0.body.wrap().into_llvm(context)?;
        if !context.is_basic_block_terminated() {
            context
                .build_unconditional_branch(context.current_function().borrow().return_block())?;
        }

        context.set_basic_block(context.current_function().borrow().return_block());
        match context.current_function().borrow().r#return() {
            solx_codegen_evm::FunctionReturn::None => {
                context.build_return(None)?;
            }
            solx_codegen_evm::FunctionReturn::Primitive { pointer } => {
                let return_value = context.build_load(pointer, "return_value")?;
                context.build_return(Some(&return_value))?;
            }
            solx_codegen_evm::FunctionReturn::Compound { pointer, .. } => {
                let return_value = context.build_load(pointer, "return_value")?;
                context.build_return(Some(&return_value))?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
///
/// This module contains only dialect-specific tests.
///
mod tests {
    use std::collections::BTreeSet;

    use crate::yul::parser::dialect::era::EraDialect;
    use solx_yul::yul::lexer::Lexer;
    use solx_yul::yul::lexer::token::location::Location;
    use solx_yul::yul::parser::error::Error;
    use solx_yul::yul::parser::statement::object::Object;

    #[test]
    fn error_invalid_attributes_single() {
        let input = r#"
object "Test" {
    code {
        {
            return(0, 0)
        }
    }
    object "Test_deployed" {
        code {
            {
                return(0, 0)
            }

            function test_$llvm_UnknownAttribute_llvm$_test() -> result {
                result := 42
            }
        }
    }
}
    "#;
        let mut invalid_attributes = BTreeSet::new();
        invalid_attributes.insert("UnknownAttribute".to_owned());

        let mut lexer = Lexer::new(input);
        let result = Object::<EraDialect>::parse(&mut lexer, None, solx_utils::CodeSegment::Deploy);
        assert_eq!(
            result,
            Err(Error::InvalidAttributes {
                location: Location::new(14, 22),
                values: invalid_attributes,
            }
            .into())
        );
    }

    #[test]
    fn error_invalid_attributes_multiple_repeated() {
        let input = r#"
object "Test" {
    code {
        {
            return(0, 0)
        }
    }
    object "Test_deployed" {
        code {
            {
                return(0, 0)
            }

            function test_$llvm_UnknownAttribute1_UnknownAttribute1_UnknownAttribute2_llvm$_test() -> result {
                result := 42
            }
        }
    }
}
    "#;
        let mut invalid_attributes = BTreeSet::new();
        invalid_attributes.insert("UnknownAttribute1".to_owned());
        invalid_attributes.insert("UnknownAttribute2".to_owned());

        let mut lexer = Lexer::new(input);
        let result = Object::<EraDialect>::parse(&mut lexer, None, solx_utils::CodeSegment::Deploy);
        assert_eq!(
            result,
            Err(Error::InvalidAttributes {
                location: Location::new(14, 22),
                values: invalid_attributes,
            }
            .into())
        );
    }
}
