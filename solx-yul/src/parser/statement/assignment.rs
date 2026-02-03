//!
//! The assignment expression statement.
//!

use std::collections::BTreeSet;

use inkwell::types::BasicType;
use solx_codegen_evm::IContext;
use solx_codegen_evm::ISolidityData;

use crate::error::Error;
use crate::lexer::Lexer;
use crate::lexer::token::Token;
use crate::lexer::token::lexeme::Lexeme;
use crate::lexer::token::lexeme::symbol::Symbol;
use crate::lexer::token::location::Location;
use crate::parser::error::Error as ParserError;
use crate::parser::identifier::Identifier;
use crate::parser::statement::expression::Expression;

///
/// The Yul assignment expression statement.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct Assignment {
    /// The location.
    pub location: Location,
    /// The variable bindings.
    pub bindings: Vec<Identifier>,
    /// The initializing expression.
    pub initializer: Expression,
    /// The solc source code location.
    pub solc_location: Option<solx_utils::DebugInfoSolcLocation>,
}

impl Assignment {
    ///
    /// The element parser.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Token>) -> Result<Self, Error> {
        let mut token = crate::parser::take_or_next(initial, lexer)?;

        let solc_location =
            token
                .take_solidity_location()
                .map_err(|error| ParserError::DebugInfoParseError {
                    location: token.location,
                    details: error.to_string(),
                })?;

        let (location, identifier) = match token {
            Token {
                location,
                lexeme: Lexeme::Identifier(identifier),
                ..
            } => (location, identifier),
            token => {
                return Err(ParserError::InvalidToken {
                    location: token.location,
                    expected: vec!["{identifier}"],
                    found: token.lexeme.to_string(),
                }
                .into());
            }
        };
        let length = identifier.inner.len();

        match lexer.peek()? {
            Token {
                lexeme: Lexeme::Symbol(Symbol::Assignment),
                ..
            } => {
                lexer.next()?;

                Ok(Self {
                    location,
                    bindings: vec![Identifier::new(location, identifier.inner)],
                    initializer: Expression::parse(lexer, None)?,
                    solc_location,
                })
            }
            Token {
                lexeme: Lexeme::Symbol(Symbol::Comma),
                ..
            } => {
                let (identifiers, next) = Identifier::parse_list(
                    lexer,
                    Some(Token::new(location, Lexeme::Identifier(identifier), length)),
                )?;

                match crate::parser::take_or_next(next, lexer)? {
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Assignment),
                        ..
                    } => {}
                    token => {
                        return Err(ParserError::InvalidToken {
                            location: token.location,
                            expected: vec![":="],
                            found: token.lexeme.to_string(),
                        }
                        .into());
                    }
                }

                Ok(Self {
                    location,
                    bindings: identifiers,
                    initializer: Expression::parse(lexer, None)?,
                    solc_location,
                })
            }
            token => Err(ParserError::InvalidToken {
                location: token.location,
                expected: vec![":=", ","],
                found: token.lexeme.to_string(),
            }
            .into()),
        }
    }

    ///
    /// Get the list of unlinked deployable libraries.
    ///
    pub fn get_unlinked_libraries(&self) -> BTreeSet<String> {
        self.initializer.get_unlinked_libraries()
    }

    ///
    /// Get the list of EVM dependencies.
    ///
    pub fn accumulate_evm_dependencies(&self, dependencies: &mut solx_codegen_evm::Dependencies) {
        self.initializer.accumulate_evm_dependencies(dependencies);
    }

    ///
    /// Compiles the assignment into LLVM IR.
    ///
    pub fn into_llvm(mut self, context: &mut solx_codegen_evm::Context) -> anyhow::Result<()> {
        if let Some((solidity_data, solc_location)) = context.solidity_mut().zip(self.solc_location)
        {
            solidity_data.set_debug_info_solc_location(solc_location);
        }

        let value = match self.initializer.into_llvm(context)? {
            Some(value) => value,
            None => return Ok(()),
        };

        if self.bindings.len() == 1 {
            let identifier = self.bindings.remove(0);
            let pointer = context
                .current_function()
                .borrow()
                .get_stack_pointer(identifier.inner.as_str())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "{} Assignment to an undeclared variable `{}`",
                        identifier.location,
                        identifier.inner,
                    )
                })?;
            context.build_store(pointer, value.to_llvm())?;
            return Ok(());
        }

        let llvm_type = value.to_llvm().into_struct_value().get_type();
        let tuple_pointer = context.build_alloca(llvm_type, "assignment_pointer")?;
        context.build_store(tuple_pointer, value.to_llvm())?;

        for (index, binding) in self.bindings.into_iter().enumerate() {
            let field_pointer = context.build_gep(
                tuple_pointer,
                &[
                    context.field_const(0),
                    context
                        .integer_type(solx_utils::BIT_LENGTH_X32)
                        .const_int(index as u64, false),
                ],
                context.field_type().as_basic_type_enum(),
                format!("assignment_binding_{index}_gep_pointer").as_str(),
            )?;

            let binding_pointer = context
                .current_function()
                .borrow()
                .get_stack_pointer(binding.inner.as_str())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "{} Assignment to an undeclared variable `{}`",
                        binding.location,
                        binding.inner,
                    )
                })?;
            let value = context.build_load(
                field_pointer,
                format!("assignment_binding_{index}_value").as_str(),
            )?;
            context.build_store(binding_pointer, value)?;
        }

        Ok(())
    }
}
