//!
//! The for-loop statement.
//!

use std::collections::BTreeSet;

use solx_codegen_evm::IContext;
use solx_codegen_evm::ISolidityData;
use solx_codegen_evm::WriteLLVM;

use crate::error::Error;
use crate::lexer::Lexer;
use crate::lexer::token::Token;
use crate::lexer::token::lexeme::Lexeme;
use crate::lexer::token::lexeme::keyword::Keyword;
use crate::lexer::token::location::Location;
use crate::parser::error::Error as ParserError;
use crate::parser::statement::block::Block;
use crate::parser::statement::expression::Expression;

///
/// The Yul for-loop statement.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ForLoop {
    /// The location.
    pub location: Location,
    /// The index variables initialization block.
    pub initializer: Block,
    /// The continue condition block.
    pub condition: Expression,
    /// The index variables mutating block.
    pub finalizer: Block,
    /// The loop body.
    pub body: Block,
    /// The solc source code location.
    pub solc_location: Option<solx_utils::DebugInfoSolcLocation>,
}

impl ForLoop {
    ///
    /// The element parser.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Token>) -> Result<Self, Error> {
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
                lexeme: Lexeme::Keyword(Keyword::For),
                ..
            } => {}
            token => {
                return Err(ParserError::InvalidToken {
                    location: token.location,
                    expected: vec!["for"],
                    found: token.lexeme.to_string(),
                }
                .into());
            }
        }

        let initializer = Block::parse(lexer, None)?;

        let condition = Expression::parse(lexer, None)?;

        let finalizer = Block::parse(lexer, None)?;

        let body = Block::parse(lexer, None)?;

        Ok(Self {
            location,
            initializer,
            condition,
            finalizer,
            body,
            solc_location,
        })
    }

    ///
    /// Get the list of unlinked deployable libraries.
    ///
    pub fn get_unlinked_libraries(&self) -> BTreeSet<String> {
        let mut libraries = self.initializer.get_unlinked_libraries();
        libraries.extend(self.condition.get_unlinked_libraries());
        libraries.extend(self.finalizer.get_unlinked_libraries());
        libraries.extend(self.body.get_unlinked_libraries());
        libraries
    }

    ///
    /// Get the list of EVM dependencies.
    ///
    pub fn accumulate_evm_dependencies(&self, dependencies: &mut solx_codegen_evm::Dependencies) {
        self.initializer.accumulate_evm_dependencies(dependencies);
        self.condition.accumulate_evm_dependencies(dependencies);
        self.finalizer.accumulate_evm_dependencies(dependencies);
        self.body.accumulate_evm_dependencies(dependencies);
    }

    ///
    /// Compiles the for-loop into LLVM IR.
    ///
    pub fn into_llvm(self, context: &mut solx_codegen_evm::Context) -> anyhow::Result<()> {
        if let Some((solidity_data, solc_location)) = context.solidity_mut().zip(self.solc_location)
        {
            solidity_data.set_debug_info_solc_location(solc_location);
        }

        self.initializer.into_llvm(context)?;

        let condition_block = context.append_basic_block("for_condition");
        let body_block = context.append_basic_block("for_body");
        let increment_block = context.append_basic_block("for_increment");
        let join_block = context.append_basic_block("for_join");

        context.build_unconditional_branch(condition_block)?;
        context.set_basic_block(condition_block);
        let condition = self
            .condition
            .into_llvm(context)?
            .ok_or_else(|| anyhow::anyhow!("For-loop condition expression yielded no value"))?
            .to_llvm()
            .into_int_value();
        let condition = context.build_bit_cast_instruction(
            inkwell::builder::Builder::build_int_z_extend_or_bit_cast,
            condition,
            context.field_type(),
            "for_condition_extended",
        )?;
        let condition = context.build_int_compare(
            inkwell::IntPredicate::NE,
            condition,
            context.field_const(0),
            "for_condition_compared",
        )?;
        context.build_conditional_branch(condition, body_block, join_block)?;

        context.push_loop(body_block, increment_block, join_block);

        context.set_basic_block(body_block);
        self.body.into_llvm(context)?;
        context.build_unconditional_branch(increment_block)?;

        context.set_basic_block(increment_block);
        self.finalizer.into_llvm(context)?;
        context.build_unconditional_branch(condition_block)?;

        context.pop_loop();
        context.set_basic_block(join_block);

        Ok(())
    }
}
