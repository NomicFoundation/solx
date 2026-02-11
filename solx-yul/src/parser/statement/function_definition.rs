//!
//! The function definition statement.
//!

use std::collections::BTreeSet;

use inkwell::types::BasicType;
use solx_codegen_evm::IContext;
use solx_codegen_evm::WriteLLVM;

use crate::error::Error;
use crate::lexer::Lexer;
use crate::lexer::token::Token;
use crate::lexer::token::lexeme::Lexeme;
use crate::lexer::token::lexeme::keyword::Keyword;
use crate::lexer::token::lexeme::symbol::Symbol;
use crate::lexer::token::location::Location;
use crate::parser::attributes::get_llvm_attributes;
use crate::parser::error::Error as ParserError;
use crate::parser::identifier::Identifier;
use crate::parser::statement::block::Block;
use crate::parser::statement::expression::function_call::name::Name as FunctionName;

///
/// The function definition statement.
///
/// All functions are translated in two steps:
/// 1. The hoisted declaration
/// 2. The definition, which now has the access to all function signatures
///
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FunctionDefinition {
    /// The location.
    pub location: Location,
    /// The function identifier.
    pub identifier: String,
    /// The function formal arguments.
    pub arguments: Vec<Identifier>,
    /// The function return variables.
    pub result: Vec<Identifier>,
    /// The function body block.
    pub body: Block,
    /// The function LLVM attributes encoded in the identifier.
    pub attributes: BTreeSet<solx_codegen_evm::Attribute>,
    /// The Solidity AST node ID, if any.
    pub ast_id: Option<usize>,
}

impl FunctionDefinition {
    ///
    /// The element parser.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Token>) -> Result<Self, Error> {
        let mut token = crate::parser::take_or_next(initial, lexer)?;

        let ast_id = token
            .take_ast_id()
            .map_err(|error| ParserError::DebugInfoParseError {
                location: token.location,
                details: error.to_string(),
            })?;

        match token {
            Token {
                lexeme: Lexeme::Keyword(Keyword::Function),
                ..
            } => {}
            token => {
                return Err(ParserError::InvalidToken {
                    location: token.location,
                    expected: vec!["function"],
                    found: token.lexeme.to_string(),
                }
                .into());
            }
        }

        let (location, identifier) = match lexer.next()? {
            Token {
                lexeme: Lexeme::Identifier(identifier),
                location,
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
        let identifier = Identifier::new(location, identifier.inner);

        match FunctionName::from(identifier.inner.as_str()) {
            FunctionName::UserDefined(_) => {}
            _function_name => {
                return Err(ParserError::ReservedIdentifier {
                    location,
                    identifier: identifier.inner,
                }
                .into());
            }
        }

        match lexer.next()? {
            Token {
                lexeme: Lexeme::Symbol(Symbol::ParenthesisLeft),
                ..
            } => {}
            token => {
                return Err(ParserError::InvalidToken {
                    location: token.location,
                    expected: vec!["("],
                    found: token.lexeme.to_string(),
                }
                .into());
            }
        }

        let (arguments, next) = Identifier::parse_list(lexer, None)?;

        match crate::parser::take_or_next(next, lexer)? {
            Token {
                lexeme: Lexeme::Symbol(Symbol::ParenthesisRight),
                ..
            } => {}
            token => {
                return Err(ParserError::InvalidToken {
                    location: token.location,
                    expected: vec![")"],
                    found: token.lexeme.to_string(),
                }
                .into());
            }
        }

        let (result, next) = match lexer.peek()? {
            Token {
                lexeme: Lexeme::Symbol(Symbol::Arrow),
                ..
            } => {
                lexer.next()?;
                Identifier::parse_list(lexer, None)?
            }
            Token {
                lexeme: Lexeme::Symbol(Symbol::BracketCurlyLeft),
                ..
            } => (vec![], None),
            token => {
                return Err(ParserError::InvalidToken {
                    location: token.location,
                    expected: vec!["->", "{"],
                    found: token.lexeme.to_string(),
                }
                .into());
            }
        };

        let body = Block::parse(lexer, next)?;

        let attributes = get_llvm_attributes(&identifier)?;

        Ok(Self {
            location,
            identifier: identifier.inner,
            arguments,
            result,
            body,
            attributes,
            ast_id,
        })
    }

    ///
    /// Get the list of EVM dependencies.
    ///
    pub fn accumulate_evm_dependencies(&self, dependencies: &mut solx_codegen_evm::Dependencies) {
        self.body.accumulate_evm_dependencies(dependencies);
    }

    ///
    /// Declares the function in the LLVM IR.
    ///
    pub fn declare(&mut self, context: &mut solx_codegen_evm::Context) -> anyhow::Result<()> {
        let argument_types: Vec<_> = self
            .arguments
            .iter()
            .map(|_| context.field_type().as_basic_type_enum())
            .collect();

        let function_type = context.function_type(argument_types, self.result.len());

        context.add_function(
            self.identifier.as_str(),
            self.ast_id,
            function_type,
            self.result.len(),
            Some(inkwell::module::Linkage::Private),
        )?;

        Ok(())
    }

    ///
    /// Compiles the function into LLVM IR.
    ///
    pub fn into_llvm(mut self, context: &mut solx_codegen_evm::Context) -> anyhow::Result<()> {
        context.set_current_function(self.identifier.as_str())?;
        let r#return = context.current_function().borrow().r#return();

        context.set_basic_block(context.current_function().borrow().entry_block());
        match r#return {
            solx_codegen_evm::FunctionReturn::None => {}
            solx_codegen_evm::FunctionReturn::Primitive { pointer } => {
                let identifier = self
                    .result
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("Function return variable is missing"))?;
                context.build_store(pointer, context.field_type().const_zero())?;
                context
                    .current_function()
                    .borrow_mut()
                    .insert_stack_pointer(identifier.inner, pointer);
            }
            solx_codegen_evm::FunctionReturn::Compound { pointer, .. } => {
                for (index, identifier) in self.result.into_iter().enumerate() {
                    let r#type = context.field_type();
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
            .arguments
            .iter()
            .map(|_| context.field_type())
            .collect();
        for (index, argument) in self.arguments.iter().enumerate() {
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

        self.body.into_llvm(context)?;
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
