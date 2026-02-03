//!
//! The switch statement.
//!

pub mod case;

use std::collections::BTreeSet;

use solx_codegen_evm::IContext;
use solx_codegen_evm::ISolidityData;
use solx_codegen_evm::WriteLLVM;

use crate::yul::error::Error;
use crate::yul::lexer::Lexer;
use crate::yul::lexer::token::Token;
use crate::yul::lexer::token::lexeme::Lexeme;
use crate::yul::lexer::token::lexeme::keyword::Keyword;
use crate::yul::lexer::token::location::Location;
use crate::yul::parser::error::Error as ParserError;
use crate::yul::parser::statement::block::Block;
use crate::yul::parser::statement::expression::Expression;

use self::case::Case;

///
/// The Yul switch statement.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Switch {
    /// The location.
    pub location: Location,
    /// The expression being matched.
    pub expression: Expression,
    /// The non-default cases.
    pub cases: Vec<Case>,
    /// The optional default case, if `cases` do not cover all possible values.
    pub default: Option<Block>,
    /// The solc source code location.
    pub solc_location: Option<solx_utils::DebugInfoSolcLocation>,
}

///
/// The parsing state.
///
pub enum State {
    /// After match expression.
    CaseOrDefaultKeyword,
    /// After `case`.
    CaseBlock,
    /// After `default`.
    DefaultBlock,
}

impl Switch {
    ///
    /// The element parser.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Token>) -> Result<Self, Error> {
        let mut token = crate::yul::parser::take_or_next(initial, lexer)?;
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
                lexeme: Lexeme::Keyword(Keyword::Switch),
                ..
            } => {}
            token => {
                return Err(ParserError::InvalidToken {
                    location: token.location,
                    expected: vec!["switch"],
                    found: token.lexeme.to_string(),
                }
                .into());
            }
        }

        let mut state = State::CaseOrDefaultKeyword;
        let expression = Expression::parse(lexer, None)?;
        let mut cases = Vec::new();
        let mut default = None;

        loop {
            match state {
                State::CaseOrDefaultKeyword => match lexer.peek()? {
                    _token @ Token {
                        lexeme: Lexeme::Keyword(Keyword::Case),
                        ..
                    } => {
                        token = _token;
                        state = State::CaseBlock;
                    }
                    _token @ Token {
                        lexeme: Lexeme::Keyword(Keyword::Default),
                        ..
                    } => {
                        token = _token;
                        state = State::DefaultBlock;
                    }
                    _token => {
                        token = _token;
                        break;
                    }
                },
                State::CaseBlock => {
                    lexer.next()?;
                    cases.push(Case::parse(lexer, None)?);
                    state = State::CaseOrDefaultKeyword;
                }
                State::DefaultBlock => {
                    lexer.next()?;
                    default = Some(Block::parse(lexer, None)?);
                    break;
                }
            }
        }

        if cases.is_empty() && default.is_none() {
            return Err(ParserError::InvalidToken {
                location: token.location,
                expected: vec!["case", "default"],
                found: token.lexeme.to_string(),
            }
            .into());
        }

        Ok(Self {
            location,
            expression,
            cases,
            default,
            solc_location,
        })
    }

    ///
    /// Get the list of unlinked deployable libraries.
    ///
    pub fn get_unlinked_libraries(&self) -> BTreeSet<String> {
        let mut libraries = BTreeSet::new();
        for case in self.cases.iter() {
            libraries.extend(case.get_unlinked_libraries());
        }
        if let Some(default) = &self.default {
            libraries.extend(default.get_unlinked_libraries());
        }
        libraries
    }

    ///
    /// Get the list of EVM dependencies.
    ///
    pub fn accumulate_evm_dependencies(&self, dependencies: &mut solx_codegen_evm::Dependencies) {
        for case in self.cases.iter() {
            case.accumulate_evm_dependencies(dependencies);
        }
        if let Some(default) = &self.default {
            default.accumulate_evm_dependencies(dependencies);
        }
    }

    ///
    /// Compiles the switch into LLVM IR.
    ///
    pub fn into_llvm(self, context: &mut solx_codegen_evm::Context) -> anyhow::Result<()> {
        if let Some((solidity_data, solc_location)) = context.solidity_mut().zip(self.solc_location)
        {
            solidity_data.set_debug_info_solc_location(solc_location);
        }

        let scrutinee = self.expression.into_llvm(context)?;

        if self.cases.is_empty() {
            if let Some(block) = self.default {
                block.into_llvm(context)?;
            }
            return Ok(());
        }

        let current_block = context.basic_block();
        let join_block = context.append_basic_block("switch_join_block");

        let mut branches = Vec::with_capacity(self.cases.len());
        for (index, case) in self.cases.into_iter().enumerate() {
            let constant = case.literal.into_llvm(context)?.to_llvm();

            let expression_block = context
                .append_basic_block(format!("switch_case_branch_{}_block", index + 1).as_str());
            context.set_basic_block(expression_block);
            case.block.into_llvm(context)?;
            context.build_unconditional_branch(join_block)?;

            branches.push((constant.into_int_value(), expression_block));
        }

        let default_block = match self.default {
            Some(default) => {
                let default_block = context.append_basic_block("switch_default_block");
                context.set_basic_block(default_block);
                default.into_llvm(context)?;
                context.build_unconditional_branch(join_block)?;
                default_block
            }
            None => join_block,
        };

        context.set_basic_block(current_block);
        context.build_switch(
            scrutinee.expect("Always exists").to_llvm().into_int_value(),
            default_block,
            branches.as_slice(),
        )?;

        context.set_basic_block(join_block);

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
    fn error_invalid_token_case() {
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
                switch 42
                    branch x {}
                    default {}
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
                location: Location::new(12, 21),
                expected: vec!["case", "default"],
                found: "branch".to_owned(),
            }
            .into())
        );
    }
}
