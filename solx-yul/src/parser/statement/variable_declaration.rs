//!
//! The variable declaration statement.
//!

use std::collections::BTreeSet;

use inkwell::types::BasicType;
use inkwell::values::BasicValue;
use solx_codegen_evm::IContext;
use solx_codegen_evm::ISolidityData;

use crate::error::Error;
use crate::lexer::Lexer;
use crate::lexer::token::Token;
use crate::lexer::token::lexeme::Lexeme;
use crate::lexer::token::lexeme::keyword::Keyword;
use crate::lexer::token::lexeme::symbol::Symbol;
use crate::lexer::token::location::Location;
use crate::parser::error::Error as ParserError;
use crate::parser::identifier::Identifier;
use crate::parser::statement::expression::Expression;
use crate::parser::statement::expression::function_call::name::Name as FunctionName;

///
/// The Yul variable declaration statement.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct VariableDeclaration {
    /// The location.
    pub location: Location,
    /// The variable bindings list.
    pub bindings: Vec<Identifier>,
    /// The variable initializing expression.
    pub expression: Option<Expression>,
    /// The solc source code location.
    pub solc_location: Option<solx_utils::DebugInfoSolcLocation>,
}

impl VariableDeclaration {
    ///
    /// The element parser.
    ///
    pub fn parse(
        lexer: &mut Lexer,
        initial: Option<Token>,
    ) -> Result<(Self, Option<Token>), Error> {
        let mut token = crate::parser::take_or_next(initial, lexer)?;
        let location = token.location;

        let solc_location =
            token
                .take_solidity_location()
                .map_err(|error| ParserError::DebugInfoParseError {
                    location: token.location,
                    details: error.to_string(),
                })?;

        match token {
            Token {
                lexeme: Lexeme::Keyword(Keyword::Let),
                ..
            } => {}
            token => {
                return Err(ParserError::InvalidToken {
                    location: token.location,
                    expected: vec!["let"],
                    found: token.lexeme.to_string(),
                }
                .into());
            }
        }

        let (bindings, next) = Identifier::parse_typed_list(lexer, None)?;
        for binding in bindings.iter() {
            match FunctionName::from(binding.inner.as_str()) {
                FunctionName::UserDefined(_) => continue,
                _function_name => {
                    return Err(ParserError::ReservedIdentifier {
                        location: binding.location,
                        identifier: binding.inner.to_owned(),
                    }
                    .into());
                }
            }
        }

        match crate::parser::take_or_next(next, lexer)? {
            Token {
                lexeme: Lexeme::Symbol(Symbol::Assignment),
                ..
            } => {}
            token => {
                return Ok((
                    Self {
                        location,
                        bindings,
                        expression: None,
                        solc_location,
                    },
                    Some(token),
                ));
            }
        }

        let expression = Expression::parse(lexer, None)?;

        Ok((
            Self {
                location,
                bindings,
                expression: Some(expression),
                solc_location,
            },
            None,
        ))
    }

    ///
    /// Get the list of unlinked deployable libraries.
    ///
    pub fn get_unlinked_libraries(&self) -> BTreeSet<String> {
        self.expression
            .as_ref()
            .map_or_else(BTreeSet::new, |expression| {
                expression.get_unlinked_libraries()
            })
    }

    ///
    /// Get the list of EVM dependencies.
    ///
    pub fn accumulate_evm_dependencies(&self, dependencies: &mut solx_codegen_evm::Dependencies) {
        if let Some(expression) = self.expression.as_ref() {
            expression.accumulate_evm_dependencies(dependencies);
        }
    }

    ///
    /// Compiles the variable declaration into LLVM IR.
    ///
    pub fn into_llvm<'ctx>(
        mut self,
        context: &mut solx_codegen_evm::Context<'ctx>,
    ) -> anyhow::Result<()> {
        if let Some((solidity_data, solc_location)) = context.solidity_mut().zip(self.solc_location)
        {
            solidity_data.set_debug_info_solc_location(solc_location);
        }

        if self.bindings.len() == 1 {
            let identifier = self.bindings.remove(0);
            let r#type = identifier.r#type.unwrap_or_default().into_llvm(context);
            let pointer = context.build_alloca(r#type, identifier.inner.as_str())?;
            context
                .current_function()
                .borrow_mut()
                .insert_stack_pointer(identifier.inner.clone(), pointer);

            let value = if let Some(expression) = self.expression {
                match expression.into_llvm(context)? {
                    Some(value) => value.to_llvm(),
                    None => r#type.const_zero().as_basic_value_enum(),
                }
            } else {
                r#type.const_zero().as_basic_value_enum()
            };
            context.build_store(pointer, value)?;
            return Ok(());
        }

        for (index, binding) in self.bindings.iter().enumerate() {
            let yul_type = binding
                .r#type
                .to_owned()
                .unwrap_or_default()
                .into_llvm(context);
            let pointer = context.build_alloca(
                yul_type.as_basic_type_enum(),
                format!("binding_{index}_pointer").as_str(),
            )?;
            context.build_store(pointer, yul_type.const_zero())?;
            context
                .current_function()
                .borrow_mut()
                .insert_stack_pointer(binding.inner.to_owned(), pointer);
        }

        let expression = match self.expression.take() {
            Some(expression) => expression,
            None => return Ok(()),
        };
        let location = expression.location();
        let expression = match expression.into_llvm(context)? {
            Some(expression) => expression,
            None => return Ok(()),
        };

        let llvm_type = context.structure_type(
            self.bindings
                .iter()
                .map(|binding| {
                    binding
                        .r#type
                        .to_owned()
                        .unwrap_or_default()
                        .into_llvm(context)
                        .as_basic_type_enum()
                })
                .collect::<Vec<inkwell::types::BasicTypeEnum<'ctx>>>()
                .as_slice(),
        );
        if expression.value.get_type() != llvm_type.as_basic_type_enum() {
            anyhow::bail!(
                "{location} Assignment to {:?} received an invalid number of arguments",
                self.bindings
            );
        }
        let pointer = context.build_alloca(llvm_type, "bindings_pointer")?;
        context.build_store(pointer, expression.to_llvm())?;

        for (index, binding) in self.bindings.into_iter().enumerate() {
            let pointer = context.build_gep(
                pointer,
                &[
                    context.field_const(0),
                    context
                        .integer_type(solx_utils::BIT_LENGTH_X32)
                        .const_int(index as u64, false),
                ],
                binding.r#type.unwrap_or_default().into_llvm(context),
                format!("binding_{index}_gep_pointer").as_str(),
            )?;

            let value = context.build_load(pointer, format!("binding_{index}_value").as_str())?;
            let pointer = context
                .current_function()
                .borrow_mut()
                .get_stack_pointer(binding.inner.as_str())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "{} Assignment to an undeclared variable `{}`",
                        binding.location,
                        binding.inner
                    )
                })?;
            context.build_store(pointer, value)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::lexer::token::location::Location;
    use crate::parser::error::Error;
    use crate::parser::statement::object::Object;

    #[test]
    fn unreserved_keyword() {
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
                let object := 42
                let code := 84
                return(0, 0)
            }
        }
    }
}
    "#;

        let mut lexer = Lexer::new(input);
        let result = Object::parse(&mut lexer, None, solx_utils::CodeSegment::Deploy);
        assert!(result.is_ok());
    }

    #[test]
    fn error_reserved_identifier() {
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
                let basefee := 42
                return(0, 0)
            }
        }
    }
}
    "#;

        let mut lexer = Lexer::new(input);
        let result = Object::parse(&mut lexer, None, solx_utils::CodeSegment::Deploy);
        assert_eq!(
            result,
            Err(Error::ReservedIdentifier {
                location: Location::new(11, 21),
                identifier: "basefee".to_owned()
            }
            .into())
        );
    }
}
