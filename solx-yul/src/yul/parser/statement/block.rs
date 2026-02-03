//!
//! The source code block.
//!

use std::collections::BTreeSet;

use solx_codegen_evm::IContext;
use solx_codegen_evm::ISolidityData;

use crate::yul::error::Error;
use crate::yul::lexer::Lexer;
use crate::yul::lexer::token::Token;
use crate::yul::lexer::token::lexeme::Lexeme;
use crate::yul::lexer::token::lexeme::symbol::Symbol;
use crate::yul::lexer::token::location::Location;
use crate::yul::parser::error::Error as ParserError;
use crate::yul::parser::statement::Statement;
use crate::yul::parser::statement::assignment::Assignment;
use crate::yul::parser::statement::expression::Expression;

///
/// The Yul source code block.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Block {
    /// The location.
    pub location: Location,
    /// The block statements.
    pub statements: Vec<Statement>,
    /// The solc source code location.
    pub solc_location: Option<solx_utils::DebugInfoSolcLocation>,
    /// The solc source code location before the end of the block.
    pub end_solc_location: Option<solx_utils::DebugInfoSolcLocation>,
}

impl Block {
    ///
    /// The element parser.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Token>) -> Result<Self, Error> {
        let mut token = crate::yul::parser::take_or_next(initial, lexer)?;

        let solc_location =
            token
                .take_solidity_location()
                .map_err(|error| ParserError::DebugInfoParseError {
                    location: token.location,
                    details: error.to_string(),
                })?;

        let mut statements = Vec::new();

        let location = match token {
            Token {
                lexeme: Lexeme::Symbol(Symbol::BracketCurlyLeft),
                location,
                ..
            } => location,
            token => {
                return Err(ParserError::InvalidToken {
                    location: token.location,
                    expected: vec!["{"],
                    found: token.lexeme.to_string(),
                }
                .into());
            }
        };

        let mut remaining = None;
        let end_solc_location;

        loop {
            match crate::yul::parser::take_or_next(remaining.take(), lexer)? {
                token @ Token {
                    lexeme: Lexeme::Keyword(_),
                    ..
                } => {
                    let (statement, next) = Statement::parse(lexer, Some(token))?;
                    remaining = next;
                    statements.push(statement);
                }
                token @ Token {
                    lexeme: Lexeme::Literal(_),
                    ..
                } => {
                    statements
                        .push(Expression::parse(lexer, Some(token)).map(Statement::Expression)?);
                }
                token @ Token {
                    lexeme: Lexeme::Identifier(_),
                    ..
                } => match lexer.peek()? {
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Assignment),
                        ..
                    } => {
                        statements.push(
                            Assignment::parse(lexer, Some(token)).map(Statement::Assignment)?,
                        );
                    }
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Comma),
                        ..
                    } => {
                        statements.push(
                            Assignment::parse(lexer, Some(token)).map(Statement::Assignment)?,
                        );
                    }
                    _ => {
                        statements.push(
                            Expression::parse(lexer, Some(token)).map(Statement::Expression)?,
                        );
                    }
                },
                token @ Token {
                    lexeme: Lexeme::Symbol(Symbol::BracketCurlyLeft),
                    ..
                } => statements.push(Block::parse(lexer, Some(token)).map(Statement::Block)?),
                mut token @ Token {
                    lexeme: Lexeme::Symbol(Symbol::BracketCurlyRight),
                    ..
                } => {
                    end_solc_location = token.take_solidity_location().map_err(|error| {
                        ParserError::DebugInfoParseError {
                            location: token.location,
                            details: error.to_string(),
                        }
                    })?;
                    break;
                }
                token => {
                    return Err(ParserError::InvalidToken {
                        location: token.location,
                        expected: vec!["{keyword}", "{expression}", "{identifier}", "{", "}"],
                        found: token.lexeme.to_string(),
                    }
                    .into());
                }
            }
        }

        Ok(Self {
            location,
            statements,
            solc_location,
            end_solc_location,
        })
    }

    ///
    /// Get the list of unlinked deployable libraries.
    ///
    pub fn get_unlinked_libraries(&self) -> BTreeSet<String> {
        let mut libraries = BTreeSet::new();
        for statement in self.statements.iter() {
            libraries.extend(statement.get_unlinked_libraries());
        }
        libraries
    }

    ///
    /// Get the list of EVM dependencies.
    ///
    pub fn accumulate_evm_dependencies(&self, dependencies: &mut solx_codegen_evm::Dependencies) {
        for statement in self.statements.iter() {
            statement.accumulate_evm_dependencies(dependencies);
        }
    }
}

impl solx_codegen_evm::WriteLLVM for Block {
    fn into_llvm(self, context: &mut solx_codegen_evm::Context) -> anyhow::Result<()> {
        let current_function = context.current_function().borrow().name().to_owned();
        let current_block = context.basic_block();

        let mut functions = Vec::with_capacity(self.statements.len());
        let mut local_statements = Vec::with_capacity(self.statements.len());
        for statement in self.statements.into_iter() {
            match statement {
                Statement::FunctionDefinition(statement) => functions.push(statement),
                statement => local_statements.push(statement),
            }
        }
        for function in functions.iter_mut() {
            function.declare(context)?;
        }
        for function in functions.into_iter() {
            function.into_llvm(context)?;
        }

        context.set_current_function(current_function.as_str())?;
        context.set_basic_block(current_block);

        if let Some((solidity_data, solc_location)) = context.solidity_mut().zip(self.solc_location)
        {
            solidity_data.set_debug_info_solc_location(solc_location);
        }

        for statement in local_statements.into_iter() {
            if context.basic_block().get_terminator().is_some() {
                break;
            }

            match statement {
                Statement::Block(block) => {
                    block.into_llvm(context)?;
                }
                Statement::Expression(expression) => {
                    expression.into_llvm(context)?;
                }
                Statement::VariableDeclaration(statement) => statement.into_llvm(context)?,
                Statement::Assignment(statement) => statement.into_llvm(context)?,
                Statement::IfConditional(statement) => statement.into_llvm(context)?,
                Statement::Switch(statement) => statement.into_llvm(context)?,
                Statement::ForLoop(statement) => statement.into_llvm(context)?,
                Statement::Continue(statement) => {
                    if let Some((solidity_data, solc_location)) =
                        context.solidity_mut().zip(statement.solc_location)
                    {
                        solidity_data.set_debug_info_solc_location(solc_location);
                    }

                    context.build_unconditional_branch(context.r#loop().continue_block)?;
                    break;
                }
                Statement::Break(statement) => {
                    if let Some((solidity_data, solc_location)) =
                        context.solidity_mut().zip(statement.solc_location)
                    {
                        solidity_data.set_debug_info_solc_location(solc_location);
                    }

                    context.build_unconditional_branch(context.r#loop().join_block)?;
                    break;
                }
                Statement::Leave(statement) => {
                    if let Some((solidity_data, solc_location)) =
                        context.solidity_mut().zip(statement.solc_location)
                    {
                        solidity_data.set_debug_info_solc_location(solc_location);
                    }

                    context.build_unconditional_branch(
                        context.current_function().borrow().return_block(),
                    )?;
                    break;
                }
                statement => anyhow::bail!(
                    "{} Unexpected local statement: {statement:?}",
                    statement.location(),
                ),
            }
        }

        if let Some((solidity_data, solc_location)) =
            context.solidity_mut().zip(self.end_solc_location)
        {
            solidity_data.set_debug_info_solc_location(solc_location);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::yul::lexer::Lexer;
    use crate::yul::lexer::token::location::Location;
    use crate::yul::parser::error::Error;
    use crate::yul::parser::statement::object::Object;

    #[test]
    fn error_invalid_token_bracket_curly_left() {
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
                (
                    return(0, 0)
                }
            }
        }
    }
}
    "#;

        let mut lexer = Lexer::new(input);
        let result = Object::parse(&mut lexer, None, solx_utils::CodeSegment::Deploy);
        assert_eq!(
            result,
            Err(Error::InvalidToken {
                location: Location::new(11, 17),
                expected: vec!["{keyword}", "{expression}", "{identifier}", "{", "}"],
                found: "(".to_owned(),
            }
            .into())
        );
    }

    #[test]
    fn error_invalid_token_statement() {
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
                :=
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
            Err(Error::InvalidToken {
                location: Location::new(11, 17),
                expected: vec!["{keyword}", "{expression}", "{identifier}", "{", "}"],
                found: ":=".to_owned(),
            }
            .into())
        );
    }
}
